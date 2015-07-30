#![feature(hashmap_hasher)]
#![feature(reflect_marker)]
#![feature(test)]
extern crate test as bench;

#[macro_use]
extern crate mopa;
extern crate num;

// Re-export
#[macro_use]
mod macros;

pub use self::store::StoreValue;
pub use self::store::Cast;
pub use self::context::DefaultContextManager;
pub use self::lookup::PropertyAccessor;

//mod error;
pub mod context;
mod store;
mod lookup;

/// Key trait to create a model that support two-ways databindings
/// with oil.
///
/// The simplest way to implement it, is by using
/// the `declare_data_binding!` macro like this:
///
/// ```rust
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
/// The trait `Store` is [mopafied](https://github.com/chris-morgan/mopa)
/// to allow cast(s) to the original type:
///
/// ```rust
/// # #[macro_use]
/// # extern crate oil_databindings;
/// # use oil_databindings::{PropertyAccessor, StoreValue, Store};
/// # use oil_databindings::context::AmbientModel;
/// # use oil_databindings::context::Context;
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
///     let p = Box::new(Player {
///         name: "Bob".to_string(),
///         pv: 10,
///         xp: 0,
///         non_relevant_stuff: 0
///     });
///     let mut a = AmbientModel::default();
///
///     // Registering the player object (giving ownership here)
///     a.register_store("player".to_string(), p);
///
///     // Checking that the player is correctly registered:
///     {
///         let attribute = a.get_attribute(PropertyAccessor::new("player.name")).unwrap();
///         match attribute {
///             StoreValue::String(s) => println!("Hello {}!", s),
///             _ => unreachable!(),
///         }
///     }
///
///     // Get back a mutable reference to the player instance
///     let iam_bob = a.get_store_mut("player".to_string()).unwrap()
///                    .downcast_ref::<Box<Player>>().unwrap();
///     assert_eq!(iam_bob.name, "Bob");
///     assert_eq!(iam_bob.pv, 10);
/// # }
/// ```
///
pub trait Store: mopa::Any {

    /// Return the value corresponding to the key 'k'.
    /// If no value is found with such a name, the trait
    /// implementer should returns `None`.
    fn get_attribute<'a>(&'a self, k: PropertyAccessor) -> AttributeGetResult<'a>;

    /// This method set the value for the attribute named 'k'.
    /// For consistency the lookup algorithm should be the
    /// same as with `get_attribute`.
    fn set_attribute<'a>(&mut self, k: PropertyAccessor, value: StoreValue<'a>) -> AttributeSetResult<'a>;
}

mopafy!(Store);

/// Result type when calling `get_attribute` on a `Store`
/// object.
pub enum AttributeGetResult<'a> {
    /// This value is returned when the get has succeeded
    /// and the value is a `PrimitiveType`.
    PrimitiveType(StoreValue<'a>),
    /// This value is returned when the get has succeeded
    /// and the value is a `IterableType`.
    IterableType(Box<Iterator<Item=&'a Store> + 'a>),
    /// This value is returned to indicate that there's no such property
    /// accessible for the given PropertyAccessor.
    NoSuchProperty,
}

/// Result type when calling `get_attribute_mut` on a `Store`
/// object.
pub enum AttributeMutResult<'a> {
    /// This value should be returned when the get has succeeded
    /// and the value is  a `PrimitiveType`.
    PrimitiveType(&'a mut store::AssignFromCast),
    /// This value should be returned when the get has succeeded
    /// and the value is an `IterableType`.
    IterableType(Box<Iterator<Item=&'a mut Store> + 'a>),
    /// Access didn't provide any result.
    NoSuchProperty
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
//                   IMPLS                  //
// ======================================== //

impl<'a> AttributeGetResult<'a> {

    pub fn unwrap(self) -> StoreValue<'a> {
        match self {
            AttributeGetResult::PrimitiveType(s) => s,
            _ => panic!(),
        }
    }

    pub fn unwrap_iter(self) -> Box<Iterator<Item=&'a Store> + 'a> {
        match self {
            AttributeGetResult::IterableType(it) => it,
            _ => panic!(),
        }
    }

    pub fn is_found(&self) -> bool {
        match self {
            &AttributeGetResult::PrimitiveType(_) => true,
            &AttributeGetResult::IterableType(_) => true,
            _ => false
        }
    }
}

// ======================================== //
//                   TESTS                  //
// ======================================== //

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug)]
    struct Player {
        name: String,
        test: Vec<String>,
        pv: i64,
        xp: i64,
        non_relevant_stuff: usize,
    }

    impl Player {
        fn new<T: ToString>(name: T, pv: i64, xp: i64) -> Player {
            Player {
                name: name.to_string(),
                test: Vec::new(),
                pv: pv,
                xp: xp,
                non_relevant_stuff: 0,
            }
        }
    }

    declare_data_binding! {
        Player {
            name,
            test,
            pv,
            xp
        }
    }

    #[test]
    fn register_global_player() {
        let mut context = DefaultContextManager::default();
        let player = Player::new("Grub", 42, 100);
        context.register_global_store("player".to_string(), player);
        assert_eq!(context.get_attribute("main", "player.pv").unwrap(), StoreValue::Integer(42));
        assert_eq!(context.get_attribute("main", "player.xp").unwrap(), StoreValue::Integer(100));
    }

    #[test]
    fn register_global_value() {
        let mut context = DefaultContextManager::default();
        context.register_global_value("option.width".to_string(), 42);
        assert_eq!(context.get_attribute("main", "option.width").unwrap(), StoreValue::Integer(42));
    }

    // TODO(Nemikolh): Move this test in manager.rs
    #[test]
    fn lookup_should_pick_store_before_global() {
        let mut context = DefaultContextManager::default();
        context.register_global_value("player.pv".to_string(), 12);
        assert_eq!(context.get_attribute("main", "player.pv").unwrap(), StoreValue::Integer(12));
        let player = Player::new("Grub", 42, 100);
        context.register_global_store("player".to_string(), player);
        assert_eq!(context.get_attribute("main", "player.pv").unwrap(), StoreValue::Integer(42));
    }

}
