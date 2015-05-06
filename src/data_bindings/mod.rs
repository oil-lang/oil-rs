pub use self::buffer::DataBindingBuffer;

use std::rc::{Rc,Weak};
use std::collections::HashMap;
use std::collections::hash_state::HashState;
use std::cell::RefCell;
use std::error::Error;

use router::Router;

use self::DataBindingError::*;

mod buffer;

pub type IteratingClosure<'b> = for <'a> FnMut(&mut Iterator<Item=&'a mut DBStore>) + 'b;

pub type BindingResult<T> = Result<T,DataBindingError>;

macro_rules! declare_data_binding {
    ($name:ident {
        $($field:ident:$type_field:ty),*
    }) => (
        declare_data_binding! {
            $name {
                $($field -> $field: $type_field),*
            }
        }
        );
    ($name:ident {
        $($key:ident -> $field:ident : $type_field:ty),*
    }) => (
        impl DBStore for $name {
            fn get_value(&self, k: &str) -> Option<StoreValue> {
                match k {
                    $(stringify!($key) => Some(StoreValue::from(self.$field.clone())),)*
                    _ => None,
                }
            }
            fn set_value(&mut self, k: &str, value: StoreValue) -> Option<StoreValue> {
                match k {
                    $(stringify!($key) => {
                        self.$field = value.into();
                        None
                    })*
                    _ => Some(value),
                }
            }
        }

        impl BulkGet for [$name] {
            fn compare_and_update(&self, k: &str, output: &mut Vec<StoreValue>) -> BindingResult<bool> {
                match k {
                    $(stringify!($key) => {
                        let mut has_changed = false;
                        if output.len() != self.len() {
                            has_changed = true;
                            if output.len() > self.len() {
                                output.truncate(self.len());
                            } else {
                                let len = output.len();
                                output.reserve(self.len() - len);
                            }
                        }
                        for (element, old_value) in self.iter().zip(output.iter_mut()) {
                            let new_value: StoreValue = element.$field.clone().into();
                            if new_value != *old_value {
                                has_changed = true;
                                *old_value = new_value;
                            }
                        }
                        for input in self.iter().skip(output.len()) {
                            (*output).push(input.$field.clone().into());
                        }
                        Ok(has_changed)
                    })*
                    _ => Err(DataBindingError::KeyNotFound(format!(": {} for type {}", k, stringify!($name)))),
                }
            }
        }

        )
}

#[derive(Clone,Eq,PartialEq,Debug)]
pub enum DataBindingError {
    DanglingReference(String),
    IteratorNotFound(String),
    KeyNotFound(String),
    ViewNotFound(String),
}

impl Error for DataBindingError {
    fn description(&self) -> &str {
        match *self {
            DanglingReference(_) => "Dangling data binding reference",
            IteratorNotFound(_) => "Repeat iterator not found",
            KeyNotFound(_) => "Key not found",
            ViewNotFound(_) => "View not found",
        }
    }
}

impl ::std::fmt::Display for DataBindingError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        try!(self.description().fmt(f));
        let details = match *self {
            DanglingReference(ref s) => s,
            IteratorNotFound(ref s) => s,
            KeyNotFound(ref s) => s,
            ViewNotFound(ref s) => s,
        };
        details.fmt(f)
    }
}

// TODO: Implement Eq and PartialEq manually to compare Strings and Integers
#[derive(Clone,Debug,Eq,PartialEq)]
pub enum StoreValue {
    String(String),
    Integer(i64),
}

impl From<i64> for StoreValue {
    fn from(i: i64) -> StoreValue {
        StoreValue::Integer(i)
    }
}

impl Into<i64> for StoreValue {
    fn into(self) -> i64 {
        use std::str::FromStr;
        match self {
            StoreValue::String(data) => {
                match i64::from_str(&data) {
                    Ok(i) => i,
                    Err(error) => {
                        println!("ERROR: Parsing to integer error {}", error);
                        0
                    }
                }
            }
            StoreValue::Integer(i) => i,
        }
    }
}

impl From<String> for StoreValue {
    fn from(s: String) -> StoreValue {
        StoreValue::String(s)
    }
}

impl Into<String> for StoreValue {
    fn into(self) -> String {
        match self {
            StoreValue::String(data) => {
                data
            }
            StoreValue::Integer(i) => i.to_string(),
        }
    }
}


pub trait BulkGet {
    fn compare_and_update(&self, k: &str, output: &mut Vec<StoreValue>) -> BindingResult<bool>;
}

pub trait DBStore {
    fn get_value(&self, k: &str) -> Option<StoreValue>;
    fn set_value(&mut self, k: &str, value: StoreValue) -> Option<StoreValue>;
}

impl <S> DBStore for HashMap<String,StoreValue,S>
where S: HashState {
    fn get_value(&self, k: &str) -> Option<StoreValue> {
        self.get(k).cloned()
    }

    fn set_value(&mut self, k: &str, value: StoreValue) -> Option<StoreValue> {
        match self.get_mut(k) {
            None => Some(value),
            Some(entry) => {
                *entry = value;
                None
            }
        }
    }
}

impl <T> DBStore for [T]
where T: DBStore {
    fn get_value(&self, k: &str) -> Option<StoreValue> {
        for i in self.iter().rev() {
            let value = i.get_value(k);
            if value.is_some() {
                return value;
            }
        }
        None
    }

    fn set_value(&mut self, k: &str, mut value: StoreValue) -> Option<StoreValue> {
        for i in self.iter_mut().rev() {
            match i.set_value(k, value) {
                None => return None,
                Some(ret) => value = ret,
            }
        }
        Some(value)
    }
}

impl <'a> DBStore for Box<DBStore + 'a> {
    fn get_value(&self, k: &str) -> Option<StoreValue> {
        (**self).get_value(k)
    }

    fn set_value(&mut self, k: &str, value: StoreValue) -> Option<StoreValue> {
        (**self).set_value(k, value)
    }
}

#[derive(Default)]
struct DataBinderScope {
    values: HashMap<String,StoreValue>,
    stores: HashMap<String,Box<DBStore>>,
    iterators: HashMap<String,Box<IsRepeatable>>,
}

impl DBStore for DataBinderScope {
    fn get_value(&self, k: &str) -> Option<StoreValue> {
        for (prefix, key) in PrefixKeyIterator::new(k) {
            if let Some(store) = self.stores.get(prefix) {
                // If we have a store registered, we look here first
                let result = store.get_value(key);
                if result.is_some() {
                    return result;
                }
            }
        }
        self.values.get_value(k)
    }

    fn set_value(&mut self, k: &str, mut value: StoreValue) -> Option<StoreValue> {
        for (prefix, key) in PrefixKeyIterator::new(k) {
            if let Some(store) = self.stores.get_mut(prefix) {
                // If we have a store registered, we look here first
                match store.set_value(key, value) {
                    None => return None,
                    Some(ret) => value = ret,
                }
            }
        }
        self.values.set_value(k, value)
    }
}

impl DataBinderScope {
    fn register_value(&mut self, key: String, value: StoreValue) -> Result<(),StoreValue> {
        match self.values.insert(key, value) {
            Some(old) => Err(old),
            None => Ok(()),
        }
    }

    fn register_store(&mut self, prefix: String, store: Box<DBStore + 'static>)
        -> Result<(),Box<DBStore + 'static>> {
        match self.stores.insert(prefix, store) {
            Some(old) => Err(old),
            None => Ok(()),
        }
    }

    fn register_iterator(&mut self, prefix: String, iterable: Box<IsRepeatable + 'static>)
        -> Result<(),Box<IsRepeatable + 'static>> {
        match self.iterators.insert(prefix, iterable) {
            Some(old) => Err(old),
            None => Ok(()),
        }
    }

    fn iter(&self, k: &str, closure: &mut IteratingClosure) -> bool {
        match self.iterators.get(k) {
            None => return false,
            Some(it) => {
                it.iter(closure)
            }
        }
    }

    fn compare_and_update(&self, iterator: &str, key: &str, output: &mut Vec<StoreValue>) -> BindingResult<bool> {
        match self.iterators.get(iterator) {
            None => Err(DataBindingError::IteratorNotFound(format!(": {}", iterator))),
            Some(it) => it.compare_and_update(key, output),
        }
    }

    fn iterator_len(&self, iterator: &str) -> BindingResult<u32> {
        match self.iterators.get(iterator) {
            None => Err(DataBindingError::IteratorNotFound(format!(": {}", iterator))),
            Some(it) => it.len(),
        }
    }
}

#[derive(Default)]
pub struct DataBinderContext {
    global: DataBinderScope,
    views: HashMap<String,DataBinderScope>,
    current_view: String,
}

impl DataBinderContext {
    pub fn new(router: &Router) -> DataBinderContext {
        let mut binder = DataBinderContext::default();
        binder.register_views(router);
        binder
    }

    pub fn register_views(&mut self, router: &Router) {
        for name in router.iter_name_views() {
            self.register_view(name.clone());
        }
    }

    pub fn switch_to_view(&mut self, view: String) {
        self.current_view = view;
    }

    pub fn register_global_value(&mut self, key: String, value: StoreValue) {
        if let Err(old) = self.global.register_value(key.clone(), value) {
            println!("WARNING: re-registering global value {} (old value {:?})", key, old);
        }
    }

    pub fn register_global_store<T>(&mut self, prefix: String, value: &Rc<RefCell<T>>)
    where T: DBStore + 'static {
        let v = Box::new(Proxy::new(value));
        if let Err(_) = self.global.register_store(prefix.clone(), v) {
            println!("WARNING: overriding global object {}", prefix);
        }
    }

    pub fn register_value(&mut self, view: &str, key: String, value: StoreValue) -> BindingResult<()> {
        match self.views.get_mut(view) {
            None => Err(DataBindingError::ViewNotFound(format!(": {}", view))),
            Some(view_scope) => {
                if let Err(old) = view_scope.register_value(key.clone(), value) {
                    // Don't throw an error, just print a warning
                    println!("WARNING: View {}: re-registering value {} (old value {:?})", view, key, old);
                }
                Ok(())
            }
        }
    }

    pub fn register_store<T>(&mut self, view: &str, prefix: String, value: &Rc<RefCell<T>>) -> BindingResult<()>
    where T: DBStore + 'static {
        match self.views.get_mut(view) {
            None => Err(DataBindingError::ViewNotFound(format!(": {}", view))),
            Some(view_scope) => {
                let v = Box::new(Proxy::new(value));
                if let Err(_) = view_scope.register_store(prefix.clone(), v) {
                    // Don't throw an error, just print a warning
                    println!("WARNING: View {}: overriding object {}", view, prefix);
                }
                Ok(())
            }
        }
    }

    pub fn register_iterator<T>(&mut self, view: &str, key: String, iterator: &Rc<RefCell<Vec<T>>>) -> BindingResult<()>
    where T: DBStore + 'static,
          [T]: BulkGet {
        match self.views.get_mut(view) {
            None => Err(DataBindingError::ViewNotFound(format!(": {}", view))),
            Some(view_scope) => {
                let v = Box::new(RepeatProxy::new(iterator));
                if let Err(_) = view_scope.register_iterator(key.clone(), v) {
                    // Don't throw an error, just print a warning
                    println!("WARNING: View {}: overriding iterator {}", view, key);
                }
                Ok(())
            }
        }
    }

    pub fn register_global_iterator<T>(&mut self, key: String, iterator: &Rc<RefCell<Vec<T>>>)
    where T: DBStore + 'static,
          [T]: BulkGet {
        let v = Box::new(RepeatProxy::new(iterator));
        if let Err(_) = self.global.register_iterator(key.clone(), v) {
            println!("WARNING: re-registering global iterator {}", key);
        }
    }

    fn register_view(&mut self, view: String) {
        self.views.entry(view).or_insert_with(|| DataBinderScope::default());
    }
}

// Private trait for UIL
pub trait DBCLookup {
    fn get_value(&self, k: &str) -> Option<StoreValue>;
    fn set_value(&mut self, k: &str, value: StoreValue);
    fn iter(&self, k: &str, closure: &mut IteratingClosure) -> bool;
    fn compare_and_update(&self, iterator: &str, k: &str, output: &mut Vec<StoreValue>) -> BindingResult<bool>;
    fn iterator_len(&self, iterator: &str) -> BindingResult<u32>;
}

impl DBCLookup for DataBinderContext {
    fn get_value(&self, k: &str) -> Option<StoreValue> {
        match self.views.get(&self.current_view) {
            None => {
                println!("WARNING: Did not find view {}", &self.current_view);
            }
            Some(view_store) => {
                let result = view_store.get_value(k);
                if result.is_some() {
                    return result;
                }
            }
        }
        // Did not find view, or view did not have the corresponding value
        self.global.get_value(k)
    }

    fn set_value(&mut self, k: &str, value: StoreValue) {
        match self.views.get_mut(&self.current_view) {
            None => {
                println!("WARNING: Did not find view {}", &self.current_view);
            }
            Some(view_store) => {
                let result = view_store.set_value(k, value);
                match result {
                    None => {},
                    Some(value) => {
                        self.global.set_value(k, value);
                    }
                }
            }
        }
    }

    fn iter(&self, k: &str, closure: &mut IteratingClosure) -> bool {
        if let Some(view_scope) = self.views.get(&self.current_view) {
            if view_scope.iter(k, closure) {
                return true;
            }
        }
        self.global.iter(k, closure)
    }

    fn compare_and_update(&self, iterator: &str, k: &str, output: &mut Vec<StoreValue>) -> BindingResult<bool> {
        if let Some(view_scope) = self.views.get(&self.current_view) {
            match view_scope.compare_and_update(iterator, k, output) {
                Err(IteratorNotFound(..)) => {} // Normal operation
                Err(e) => return Err(e),
                Ok(out) => return Ok(out),
            }
        }
        self.global.compare_and_update(iterator, k, output)
    }

    fn iterator_len(&self, iterator: &str) -> BindingResult<u32> {
        if let Some(view_scope) = self.views.get(&self.current_view) {
            match view_scope.iterator_len(iterator) {
                Err(IteratorNotFound(..)) => {} // Normal operation
                Err(e) => return Err(e),
                Ok(out) => return Ok(out),
            }
        }
        self.global.iterator_len(iterator)
    }
}

#[derive(Debug)]
struct Proxy<T: DBStore> {
    data: Weak<RefCell<T>>,
}

impl <T> DBStore for Proxy<T>
where T: DBStore {
    fn get_value(&self, k: &str) -> Option<StoreValue> {
        match self.data.upgrade() {
            None => None,
            Some(p) => p.borrow().get_value(k),
        }
    }

    fn set_value(&mut self, k: &str, value: StoreValue) -> Option<StoreValue> {
        match self.data.upgrade() {
            None => Some(value),
            Some(p) => p.borrow_mut().set_value(k, value),
        }
    }
}

impl <T: DBStore> Proxy<T> {
    fn new(value: &Rc<RefCell<T>>) -> Proxy<T> {
        Proxy {
            data: value.downgrade(),
        }
    }
}

trait IsRepeatable {
    fn iter(&self, closure: &mut IteratingClosure) -> bool;
    fn compare_and_update(&self, k: &str, output: &mut Vec<StoreValue>) -> BindingResult<bool>;
    fn len(&self) -> BindingResult<u32>;
}

pub struct RepeatProxy<T> {
    cell: Weak<RefCell<Vec<T>>>,
}

impl <T> RepeatProxy<T> {
    fn new(cell: &Rc<RefCell<Vec<T>>>) -> RepeatProxy<T> {
        RepeatProxy {
            cell: cell.downgrade(),
        }
    }
}

impl <T> IsRepeatable for RepeatProxy<T>
where T: DBStore + 'static,
      [T]: BulkGet {
    fn iter(&self, closure: &mut IteratingClosure) -> bool {
        let reference = match self.cell.upgrade() {
            Some(r) => r,
            None => return false,
        };
        let mut guard = reference.borrow_mut();
        let mut iter = guard.iter_mut().map(|item| item as &mut DBStore);
        closure(&mut iter);
        true
    }

    fn compare_and_update(&self, k: &str, output: &mut Vec<StoreValue>) -> BindingResult<bool> {
        let reference = match self.cell.upgrade() {
            Some(r) => r,
            None => return Err(DataBindingError::DanglingReference(format!(": {}", k))),
        };
        let guard = reference.borrow_mut();
        (*guard).compare_and_update(k, output)
    }

    fn len(&self) -> BindingResult<u32> {
        let reference = match self.cell.upgrade() {
            Some(r) => r,
            None => return Err(DataBindingError::DanglingReference("".to_string())),
        };
        let guard = reference.borrow();
        Ok((*guard).len() as u32)
    }
}

// An iterator over the different combinations of prefix-key
struct PrefixKeyIterator<'a> {
    data: &'a str,
    position: i8,
}

impl <'a> PrefixKeyIterator<'a> {
    fn new(data: &'a str) -> PrefixKeyIterator<'a> {
        PrefixKeyIterator {
            data: data,
            position: 0,
        }
    }
}

impl <'a> Iterator for PrefixKeyIterator<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.position == 0 {
            self.position += 1;
            return Some(("", self.data));
        }
        let mut position = self.position;
        self.position += 1;
        let mut iterator = self.data.split(|c| {
            if c == '.' {
                position -= 1;
                if position == 0 {
                    return true;
                }
            }
            false
        });
        let prefix = match iterator.next() {
            None => return None,
            Some(a) => a,
        };
        let key = match iterator.next() {
            None => return None,
            Some(a) => a,
        };
        Some((prefix,key))
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;
    use std::cell::RefCell;
    use super::*;
#[derive(Debug)]
    struct Player {
        name: String,
        pv: i64,
        xp: i64,
    }

    impl Player {
        fn new<T: ToString>(name: T, pv: i64, xp: i64) -> Player {
            Player {
                name: name.to_string(),
                pv: pv,
                xp: xp,
            }
        }

        fn new_rc<T: ToString>(name: T, pv: i64, xp: i64) -> Rc<RefCell<Player>> {
            Rc::new(RefCell::new(Player::new(name, pv, xp)))
        }
    }

    declare_data_binding! {
        Player {
            name: String,
            pv: i64,
            xp: i64
        }
    }

    #[test]
    fn register_global_player() {
        let mut context = DataBinderContext::default();
        let player = Player::new_rc("Vaelden", 42, 100);
        context.register_global_store("player".to_string(), &player);
        assert_eq!(context.get_value("player.pv").unwrap(), StoreValue::Integer(42));
        assert_eq!(context.get_value("player.xp").unwrap(), StoreValue::Integer(100));
    }

    #[test]
    fn register_global_value() {
        let mut context = DataBinderContext::default();
        context.register_global_value("option.width".to_string(),
            StoreValue::Integer(42));
        assert_eq!(context.get_value("option.width").unwrap(), StoreValue::Integer(42));
    }

    #[test]
    fn register_view_player() {
        let mut context = DataBinderContext::default();
        context.register_view("foo".to_string());
        let player = Player::new_rc("Vaelden", 42, 100);
        context.register_store("foo", "player".to_string(), &player).unwrap();

        // Not in the correct view
        assert!(context.get_value("player.pv").is_none());
        context.switch_to_view("foo".to_string());
        assert_eq!(context.get_value("player.pv").unwrap(), StoreValue::Integer(42));
        assert_eq!(context.get_value("player.xp").unwrap(), StoreValue::Integer(100));
    }

    #[test]
    fn register_view_value() {
        let mut context = DataBinderContext::default();
        context.register_view("foo".to_string());
        context.register_value("foo", "option.width".to_string(),
            StoreValue::Integer(42)).unwrap();
        assert!(context.get_value("option.width").is_none());
        context.switch_to_view("foo".to_string());
        assert_eq!(context.get_value("option.width").unwrap(), StoreValue::Integer(42));
    }

    #[test]
    fn masking_value_by_object() {
        let mut context = DataBinderContext::default();
        context.register_global_value("player.pv".to_string(),
            StoreValue::Integer(12));
        assert_eq!(context.get_value("player.pv").unwrap(), StoreValue::Integer(12));
        let player = Player::new_rc("Vaelden", 42, 100);
        context.register_global_store("player".to_string(), &player);
        assert_eq!(context.get_value("player.pv").unwrap(), StoreValue::Integer(42));
    }

    #[test]
    fn masking_global_value_by_view() {
        let foo = "foo".to_string();
        let bar = "bar".to_string();
        let mut context = DataBinderContext::default();
        context.register_view(foo.clone());
        context.register_view(bar.clone());
        context.register_view("foobar".to_string());
        context.register_value("foo", "option.width".to_string(),
            StoreValue::String("foo_value".to_string())).unwrap();
        context.register_value("bar", "option.width".to_string(),
            StoreValue::String("bar_value".to_string())).unwrap();
        context.register_global_value("option.width".to_string(),
            StoreValue::String("global_value".to_string()));

        // In view "foobar" -> get global value
        context.switch_to_view("foobar".to_string());
        assert_eq!(context.get_value("option.width").unwrap(), StoreValue::String("global_value".to_string()));
        // In view "foo" -> get foo specific value
        context.switch_to_view("foo".to_string());
        assert_eq!(context.get_value("option.width").unwrap(), StoreValue::String("foo_value".to_string()));
        // In view "bar" -> get bar specific value
        context.switch_to_view("bar".to_string());
        assert_eq!(context.get_value("option.width").unwrap(), StoreValue::String("bar_value".to_string()));
    }

    #[test]
    fn global_iterator() {
        let mut context = DataBinderContext::default();
        let players = Rc::new(RefCell::new(vec![Player::new("Vaelden", 1, 11), Player::new("Nemikolh", 2, 22)]));
        context.register_global_iterator("game.friends".to_string(), &players);
        let mut iteration = 0;
        let result = context.iter("game.friends", &mut |iterator| {
            for store in iterator {
                iteration += 1;
                match iteration {
                    1 => {
                        assert_eq!(store.get_value("pv").unwrap(), StoreValue::Integer(1));
                        assert_eq!(store.get_value("xp").unwrap(), StoreValue::Integer(11));
                    }
                    2 => {
                        assert_eq!(store.get_value("pv").unwrap(), StoreValue::Integer(2));
                        assert_eq!(store.get_value("xp").unwrap(), StoreValue::Integer(22));
                        store.set_value("xp", StoreValue::Integer(42));
                        assert_eq!(store.get_value("xp").unwrap(), StoreValue::Integer(42));
                    }
                    _ => panic!("Too many iterations"),
                }
            }
        });
        assert!(result);
        let mut result_vec = Vec::new();
        assert!(context.compare_and_update("game.friends", "pv", &mut result_vec).unwrap());
        assert_eq!(result_vec, [StoreValue::Integer(1), StoreValue::Integer(2)]);
        assert!(context.compare_and_update("game.friends", "name", &mut result_vec).unwrap());
        assert_eq!(result_vec, [StoreValue::String("Vaelden".to_string()), StoreValue::String("Nemikolh".to_string())]);
    }

    #[test]
    fn bulk_get_implementation() {
        let mut players = vec![Player::new("Vaelden", 1, 11), Player::new("Nemikolh", 2, 22)];
        let mut vec = Vec::new();
        assert!(players.compare_and_update("pv", &mut vec).unwrap());
        assert_eq!(vec, [StoreValue::Integer(1), StoreValue::Integer(2)]);
        assert!(!players.compare_and_update("pv", &mut vec).unwrap());
        assert_eq!(vec, [StoreValue::Integer(1), StoreValue::Integer(2)]);
        players.pop();
        assert!(players.compare_and_update("pv", &mut vec).unwrap());
        assert_eq!(vec, [StoreValue::Integer(1)]);
        players.push(Player::new("Cendrais", 3, 33));
        assert!(players.compare_and_update("xp", &mut vec).unwrap());
        assert_eq!(vec, [StoreValue::Integer(11), StoreValue::Integer(33)]);
    }

    #[test]
    fn invalid_iterator() {
        let mut context = DataBinderContext::default();
        let mut result_vec = Vec::new();
        let players = Rc::new(RefCell::new(vec![Player::new("Vaelden", 1, 11), Player::new("Nemikolh", 2, 22)]));
        context.register_global_iterator("game.friends".to_string(), &players);
        context.compare_and_update("invalid_id", "pv", &mut result_vec).err().unwrap(); // IteratorNotFound
        context.compare_and_update("game.friends", "invalid_key", &mut result_vec).err().unwrap(); // KeyNotFound
    }

    #[test]
    fn masking_iterator() {
        let mut context = DataBinderContext::default();
        context.register_view("foo".to_string());
        let global_players = Rc::new(RefCell::new(vec![Player::new("Vaelden", 1, 11), Player::new("Nemikolh", 2, 22)]));
        context.register_global_iterator("game.friends".to_string(), &global_players);
        let foo_players = Rc::new(RefCell::new(vec![Player::new("Cendrais", 3, 33)]));
        context.register_iterator("foo", "game.friends".to_string(), &foo_players).unwrap();
        let mut global_vec = Vec::new();
        assert!(context.compare_and_update("game.friends", "pv", &mut global_vec).unwrap());
        assert_eq!(global_vec, [StoreValue::Integer(1), StoreValue::Integer(2)]);
        context.switch_to_view("foo".to_string());
        let mut foo_vec = Vec::new();
        assert!(context.compare_and_update("game.friends", "pv", &mut foo_vec).unwrap());
        assert_eq!(foo_vec, [StoreValue::Integer(3)]);
    }
}
