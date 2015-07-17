#![feature(hashmap_hasher)]

#[macro_use]
extern crate mopa;
extern crate num;

// Re-export
#[macro_use]
mod macros;

pub use self::store::StoreValue;
pub use self::store::Cast;
pub use self::context::ContextManager;
pub use self::context::DefaultContextManager;
pub use self::lookup::PropertyAccessor;

//mod error;
mod store;
mod context;
mod lookup;

/// Key trait to create a model that support two-ways databindings
/// with oil.
///
/// The simplest way to implement it, is by using
/// the `declare_data_binding!` macro like this:
///
/// ```
/// # #[macro_use]
/// # extern crate oil_databindings;
/// struct Player {
///     name: String,
///     pv: i64,
///     xp: i64,
///     non_relevant_stuff: usize,
/// }
///
/// declare_data_binding! {
///     Player {
///         name,
///         pv,
///         xp
///     }
/// }
/// # fn main() {
/// # }
/// ```
/// 
/// `Store` is [mopafied](https://github.com/chris-morgan/mopa)
/// to allow cast(s) to the original type:
/// ```
/// # #[macro_use]
/// # extern crate oil_databindings as oil;
/// # use oil_databindings::{AmbientModel, PropertyAccessor, StoreValue};
/// # struct Player {
/// #     name: String,
/// #     pv: i64,
/// #     xp: i64,
/// #     non_relevant_stuff: usize,
/// # }
/// #
/// # declare_data_binding! {
/// #     Player {
/// #         name,
/// #         pv,
/// #         xp
/// #     }
/// # }
/// # fn main() {
///     // Creation of an AmbientModel and an instance of the previously defined store
///     let p = Box::new(Player { name: "Bob".to_string(), pv: 10, Default::default().. });
///     let mut a = AmbientModel::Default();
///
///     // Registering the player object (giving ownership here)
///     a.register_store("player", p);
///
///     // Checking that the player is correctly registered:
///     let attribute = a.get_attribute(PropertyAccessor::new("player.name")).unwrap();
///     match attribute {
///         StoreValue::String(s) => println!("Hello {}!", s),
///         _ => unreachable!(),
///     }
///
///     // Get back a mutable reference to the player instance
///     let iam_bob = a.get_store_mut("player").unwrap().downcast_ref::<Player>().unwrap();
///     assert_eq!(iam_bob.name, "Bob");
///     assert_eq!(iam_bob.pv, 10);
/// # }
/// ```
///
pub trait Store {//: mopa::Any {

    /// Return the value corresponding to the key 'k'.
    /// If no value is found with such a name, the trait
    /// implementer should returns `None`.
    fn get_attribute<'a>(&'a self, k: PropertyAccessor) -> AttributeGetResult<'a>;

    /// This method set the value for the attribute named 'k'.
    /// For consistency the lookup algorithm should be the
    /// same as with `get_attribute`.
    fn set_attribute<'a>(&mut self, k: PropertyAccessor, value: StoreValue<'a>) -> AttributeSetResult<'a>;
}

//mopafy!(Store);

/// Result type when calling `get_attribute` on a `Store`
/// object.
pub enum AttributeGetResult<'a> {
    /// This value is returned when the get has succeeded
    Found(StoreValue<'a>),
    /// This value is returned to indicate that there's no such property
    /// accessible for the given PropertyAccessor.
    NoSuchProperty,
}

impl<'a> AttributeGetResult<'a> {
    
    pub fn unwrap(self) -> StoreValue<'a> {
        match self {
            AttributeGetResult::Found(s) => s,
            _ => panic!(),
        }
    }
    
    pub fn is_found(&self) -> bool {
        match self { &AttributeGetResult::Found(_) => true, _ => false }
    }
}

/// Result type when calling `set_attribute` on a `Store`
/// object.
pub enum AttributeSetResult<'a> {
    /// This value should be returned in case of success
    /// (the value has been successfully stored)
    Stored,
    /// In case of failure due to a cast error (see trait `Cast`), return that
    /// value instead of `NoSuchProperty`, it means that the PropertyAccessor has
    /// managed to find an existing property but there was a type error.
    /// It is different from NoSuchProperty because as with Stored it stops
    /// the lookup.
    /// The main difference with `Stored` is that a warning log will be reported.
    ///
    /// TODO(Nemikolh): It could be nice if at the oil API level
    /// there was a way to configure whether or not the application
    /// should panic, log or do nothing.
    WrongType,
    /// If the lookup failed, the value argument of `set_attribute` must
    /// be returned unchanged via this enum value.
    NoSuchProperty(StoreValue<'a>)
}

// ======================================== //
//                   TESTS                  //
// ======================================== //

#[cfg(test)]
mod test {
    use std::rc::Rc;
    use std::cell::RefCell;
    use std::ops::Deref;
    use super::*;

    #[derive(Debug)]
    struct Player {
        name: String,
        pv: i64,
        xp: i64,
        non_relevant_stuff: usize,
    }

    impl Player {
        fn new<T: ToString>(name: T, pv: i64, xp: i64) -> Player {
            Player {
                name: name.to_string(),
                pv: pv,
                xp: xp,
                non_relevant_stuff: 0,
            }
        }

        fn new_rc<T: ToString>(name: T, pv: i64, xp: i64) -> Rc<RefCell<Player>> {
            Rc::new(RefCell::new(Player::new(name, pv, xp)))
        }
    }

    declare_data_binding! {
        Player {
            name,
            pv,
            xp
        }
    }

    #[test]
    fn register_global_player() {
        let mut context = ContextManager::default();
        let player = Player::new_rc("Grub", 42, 100);
        context.register_global_store("player".to_string(), &player);
        assert_eq!(context.get_attribute("player.pv").unwrap(), StoreValue::Integer(42));
        assert_eq!(context.get_attribute("player.xp").unwrap(), StoreValue::Integer(100));
    }

    #[test]
    fn register_global_value() {
        let mut context = ContextManager::default();
        context.register_global_value("option.width".to_string(), 42);
        assert_eq!(context.get_attribute("option.width").unwrap(), StoreValue::Integer(42));
    }

    #[test]
    fn masking_value_by_object() {
        let mut context = ContextManager::default();
        context.register_global_value("player.pv".to_string(), 12);
        assert_eq!(context.get_attribute("player.pv").unwrap(), StoreValue::Integer(12));
        let player = Player::new_rc("Grub", 42, 100);
        context.register_global_store("player".to_string(), &player);
        assert_eq!(context.get_attribute("player.pv").unwrap(), StoreValue::Integer(42));
    }

    #[test]
    fn global_iterator() {
        let mut context = ContextManager::default();
        let players = Rc::new(RefCell::new(vec![Player::new("Grub", 1, 11), Player::new("Gnom", 2, 22)]));
        context.register_global_iterator("game.friends".to_string(), &players);
        let result = context.iter("game.friends", &mut |iterator| {
            {
                let store = iterator.next().unwrap();
                assert_eq!(store.get_attribute(PropertyAccessor::new("pv")).unwrap(), StoreValue::Integer(1));
                assert_eq!(store.get_attribute(PropertyAccessor::new("xp")).unwrap(), StoreValue::Integer(11));
            }
            {
                let store = iterator.next().unwrap();
                assert_eq!(store.get_attribute(PropertyAccessor::new("pv")).unwrap(), StoreValue::Integer(2));
                assert_eq!(store.get_attribute(PropertyAccessor::new("xp")).unwrap(), StoreValue::Integer(22));
                store.set_value("xp", StoreValue::Integer(42));
                assert_eq!(store.get_attribute(PropertyAccessor::new("xp")).unwrap(), StoreValue::Integer(42));
            }
        });
        assert!(result);
        let mut result_vec = Vec::new();
        assert!(context.compare_and_update("game.friends", "pv", &mut result_vec).unwrap());
        assert_eq!(result_vec, [StoreValue::Integer(1), StoreValue::Integer(2)]);
        assert!(context.compare_and_update("game.friends", "name", &mut result_vec).unwrap());
        assert_eq!(result_vec, [StoreValue::String("Grub".to_string()), StoreValue::String("Gnom".to_string())]);
    }

    #[test]
    fn arrstore_implementation() {
        let mut players = vec![Player::new("Grub", 1, 11), Player::new("Gnom", 2, 22)];
        let mut vec = Vec::new();
        assert!(ArrStore::compare_and_update(players.deref(), "pv", &mut vec).unwrap());
        assert_eq!(vec, [StoreValue::Integer(1), StoreValue::Integer(2)]);
        assert!(!ArrStore::compare_and_update(players.deref(), "pv", &mut vec).unwrap());
        assert_eq!(vec, [StoreValue::Integer(1), StoreValue::Integer(2)]);
        players.pop();
        assert!(ArrStore::compare_and_update(players.deref(), "pv", &mut vec).unwrap());
        assert_eq!(vec, [StoreValue::Integer(1)]);
        players.push(Player::new("Turtle", 3, 33));
        assert!(ArrStore::compare_and_update(players.deref(), "xp", &mut vec).unwrap());
        assert_eq!(vec, [StoreValue::Integer(11), StoreValue::Integer(33)]);
    }

    #[test]
    fn invalid_iterator() {
        let mut context = ContextManager::default();
        let mut result_vec = Vec::new();
        let players = Rc::new(RefCell::new(vec![Player::new("Grub", 1, 11), Player::new("Gnom", 2, 22)]));
        context.register_global_iterator("game.friends".to_string(), &players);
        context.compare_and_update("invalid_id", "pv", &mut result_vec).err().unwrap(); // IteratorNotFound
        context.compare_and_update("game.friends", "invalid_key", &mut result_vec).err().unwrap(); // KeyNotFound
    }

}
