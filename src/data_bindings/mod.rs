pub use self::buffer::DataBindingBuffer;

use std::rc::{Rc,Weak};
use std::cell::RefCell;
use std::collections::hash_state::HashState;
use std::collections::HashMap;

// Re-export
#[macro_use]
mod macros;

pub use self::error::DataBindingError;
pub use self::error::BindingResult;
pub use self::store_value::StoreValue;
pub use self::context::DataBinderContext;

mod error;
mod store_value;
mod buffer;
mod context;
mod repeat;

pub type IteratingClosure<'b> = for <'a> FnMut(&mut Iterator<Item=&'a mut DBStore>) + 'b;

pub trait BulkGet {
    fn compare_and_update(&self, k: &str, output: &mut Vec<StoreValue>) -> BindingResult<bool>;
}

/// Key trait to create a model that support two-ways databindings
/// with oil.
///
/// The simplest way to implement it, is by using
/// the `declare_data_binding!` macro like this:
///
/// ```
/// struct Player {
///     name: String,
///     pv: i64,
///     xp: i64,
///     non_relevant_stuff: usize,
/// }
///
/// declare_data_binding! {
///     Player {
///         name: String,
///         pv: i64,
///         xp: i64
///     }
/// }
/// ```
///
pub trait DBStore {

    /// Return the value corresponding to the key 'k'.
    /// If no value is found with such a name, the trait
    /// implementer should returns `None`.
    fn get_value(&self, k: &str) -> Option<StoreValue>;

    /// This method set the value for the attribute named 'k'.
    ///
    /// **Note:**
    ///     Oil does not perform any coherence check between
    ///     get_value and set_value. It allows you to perform alternatives
    ///     checks and modifies others value if relevant.
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

trait IsRepeatable {
    fn iter(&self, closure: &mut IteratingClosure) -> bool;
    fn compare_and_update(&self, k: &str, output: &mut Vec<StoreValue>) -> BindingResult<bool>;
    fn len(&self) -> BindingResult<u32>;
}

// Private trait for OIL
pub trait DBCLookup {
    fn get_value(&self, k: &str) -> Option<StoreValue>;
    fn set_value(&mut self, k: &str, value: StoreValue);
    fn iter(&self, k: &str, closure: &mut IteratingClosure) -> bool;
    fn compare_and_update(&self, iterator: &str, k: &str, output: &mut Vec<StoreValue>) -> BindingResult<bool>;
    fn iterator_len(&self, iterator: &str) -> BindingResult<u32>;
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
