use std::rc::{Rc,Weak};
use std::collections::HashMap;
use std::collections::hash_state::HashState;
use std::cell::RefCell;

use router::Router;

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
                    $(stringify!($key) => Some(StoreValue{data: self.$field.to_string()}),)*
                    _ => None,
                }
            }
            fn set_value(&mut self, k: &str, value: StoreValue) -> Option<StoreValue> {
                use std::str::FromStr;
                match k {
                    $(stringify!($key) => {
                        match <$type_field as FromStr>::from_str(&value.data) {
                            Ok(value) => {
                                self.$field = value;
                                None
                            }
                            Err(..) => {
                                println!("WARNING: Failed to parse data binding {} as {}", &value.data, stringify!($type_field));
                                Some(value)
                            }
                        }
                    })*
                    _ => Some(value),
                }
            }
        }

        )
}

#[derive(Clone,Debug)]
pub struct StoreValue {
    data: String,
}

pub trait DBStore: ::std::fmt::Debug {
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

#[derive(Default,Debug)]
struct DataBinderScope {
    values: HashMap<String,StoreValue>,
    stores: HashMap<String,Box<DBStore>>,
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

    pub fn register_global_store<T>(&mut self, prefix: String, value: Rc<RefCell<T>>)
    where T: DBStore + 'static {
        let v = Box::new(Proxy::new(value));
        if let Err(old) = self.global.register_store(prefix.clone(), v) {
            println!("WARNING: overriding global object {} (old value {:?})", prefix, old);
        }
    }

    pub fn register_value(&mut self, view: &str, key: String, value: StoreValue) -> Result<(),String> {
        match self.views.get_mut(view) {
            None => Err(format!("Could not find view {}", view)),
            Some(view_scope) => {
                if let Err(old) = view_scope.register_value(key.clone(), value) {
                    // Don't throw an error, just print a warning
                    println!("WARNING: View {}: re-registering value {} (old value {:?})", view, key, old);
                }
                Ok(())
            }
        }
    }

    pub fn register_store<T>(&mut self, view: &str, prefix: String, value: Rc<RefCell<T>>) -> Result<(),String>
    where T: DBStore + 'static {
        match self.views.get_mut(view) {
            None => Err(format!("Could not find view {}", view)),
            Some(view_scope) => {
                let v = Box::new(Proxy::new(value));
                if let Err(old) = view_scope.register_store(prefix.clone(), v) {
                    // Don't throw an error, just print a warning
                    println!("WARNING: View {}: overriding object {} (old value {:?})", view, prefix, old);
                }
                Ok(())
            }
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
    fn new(value: Rc<RefCell<T>>) -> Proxy<T> {
        Proxy {
            data: value.downgrade(),
        }
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
        pv: usize,
        xp: usize,
    }

    declare_data_binding! {
        Player {
            pv: usize,
            xp: usize
        }
    }

    #[test]
    fn register_global_player() {
        let mut context = DataBinderContext::default();
        let player = Rc::new(RefCell::new(Player{pv: 42, xp: 100}));
        // Clone the Rc, as it will get downgraded to Weak when registered
        context.register_global_store("player".to_string(), player.clone());
        assert_eq!(context.get_value("player.pv").unwrap().data, "42");
        assert_eq!(context.get_value("player.xp").unwrap().data, "100");
    }

    #[test]
    fn register_global_value() {
        let mut context = DataBinderContext::default();
        context.register_global_value("option.width".to_string(),
            StoreValue{data: "42".to_string()});
        assert_eq!(context.get_value("option.width").unwrap().data, "42");
    }

    #[test]
    fn register_view_player() {
        let mut context = DataBinderContext::default();
        context.register_view("foo".to_string());
        let player = Rc::new(RefCell::new(Player{pv: 42, xp: 100}));
        // Clone the Rc, as it will get downgraded to Weak when registered
        context.register_store("foo", "player".to_string(), player.clone()).unwrap();

        // Not in the correct view
        assert!(context.get_value("player.pv").is_none());
        context.switch_to_view("foo".to_string());
        assert_eq!(context.get_value("player.pv").unwrap().data, "42");
        assert_eq!(context.get_value("player.xp").unwrap().data, "100");
    }

    #[test]
    fn register_view_value() {
        let mut context = DataBinderContext::default();
        context.register_view("foo".to_string());
        context.register_value("foo", "option.width".to_string(),
            StoreValue{data: "42".to_string()}).unwrap();
        assert!(context.get_value("option.width").is_none());
        context.switch_to_view("foo".to_string());
        assert_eq!(context.get_value("option.width").unwrap().data, "42");
    }

    #[test]
    fn masking_value_by_object() {
        let mut context = DataBinderContext::default();
        context.register_global_value("player.pv".to_string(),
            StoreValue{data: "12".to_string()});
        assert_eq!(context.get_value("player.pv").unwrap().data, "12");
        let player = Rc::new(RefCell::new(Player{pv: 42, xp: 100}));
        // Clone the Rc, as it will get downgraded to Weak when registered
        context.register_global_store("player".to_string(), player.clone());
        assert_eq!(context.get_value("player.pv").unwrap().data, "42");
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
            StoreValue{data: "foo_value".to_string()}).unwrap();
        context.register_value("bar", "option.width".to_string(),
            StoreValue{data: "bar_value".to_string()}).unwrap();
        context.register_global_value("option.width".to_string(),
            StoreValue{data: "global_value".to_string()});

        // In view "foobar" -> get global value
        context.switch_to_view("foobar".to_string());
        assert_eq!(context.get_value("option.width").unwrap().data, "global_value");
        // In view "foo" -> get foo specific value
        context.switch_to_view("foo".to_string());
        assert_eq!(context.get_value("option.width").unwrap().data, "foo_value");
        // In view "bar" -> get bar specific value
        context.switch_to_view("bar".to_string());
        assert_eq!(context.get_value("option.width").unwrap().data, "bar_value");
    }
}
