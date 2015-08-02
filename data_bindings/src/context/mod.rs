pub use self::manager::ContextManager;
pub use self::manager::ViewContext;
pub use self::manager::ViewContextMut;

mod manager;
//mod proxies;

//use mopa;
use std::collections::HashMap;
use store::StoreValueStatic;
use {
    Store,
    StoreValue,
    AttributeGetResult,
    AttributeMutResult,
    AttributeSetResult
};
use lookup::PropertyAccessor;

/// This trait is used to provide a possible interface for Context
/// objects managed by the `ContextManager`. It is implemented by
/// the `AmbientModel` to give an example of such a `Context`.
/// **Note:**
/// If the "Context" type for the `ContextManager` implement this trait,
/// then those function can be used also on the `ContextManager`.
pub trait Context {

    /// Register a single value at the key
    fn register_value<V: Into<StoreValueStatic>>(&mut self, key: String, value: V);

    /// Register a store:
    fn register_store<S: Store>(&mut self, key: String, store: S);

    /// Return a previously registered store:
    /// This can be useful when you want to modify an existing store but without
    /// retaining a reference to it.
    fn get_store_mut(&mut self, key: String) -> Option<&mut Box<Store + 'static>>;
}

/// Default version of the `ContextManager` where the template
/// parameter is set to `AmbientModel`.
pub type DefaultContextManager = ContextManager<AmbientModel, AmbientModel>;

/// An `AmbientModel` instance is a root object that is used
/// by the `DefaultContextManager`.
/// Internally it use a HashMap for single `StoreValue`s
/// and an other HashMap for boxed type implementing the trait `Store`.
#[derive(Default)]
pub struct AmbientModel {
    values: HashMap<String, StoreValueStatic>,
    stores: HashMap<String, Box<Store>>,
}

/// Minimal contraint to be used in a `ContextManager`:
/// implement the trait `Store`.
impl Store for AmbientModel {

    fn get_attribute<'a>(&'a self, k: PropertyAccessor) -> AttributeGetResult<'a> {
        let value = self.stores.get_attribute(k.clone());
        if value.is_found() {
            value
        } else {
            self.values.get_attribute(k)
        }
    }

    fn get_attribute_mut<'a>(&'a mut self, k: PropertyAccessor) -> AttributeMutResult<'a> {
        let value = self.stores.get_attribute_mut(k.clone());
        if value.is_found() {
            value
        } else {
            self.values.get_attribute_mut(k)
        }
    }

    fn set_attribute<'a>(&mut self, k: PropertyAccessor, value: StoreValue<'a>) -> AttributeSetResult<'a> {
        match self.stores.set_attribute(k.clone(), value) {
            AttributeSetResult::NoSuchProperty(v) => {
                self.values.set_attribute(k, v)
            }
            _ => AttributeSetResult::Stored
        }
    }
}

// Context implementation
impl Context for AmbientModel {

    fn register_value<V: Into<StoreValueStatic>>(&mut self, key: String, value: V) {
        self.values.insert(key, value.into());
    }

    fn register_store<S: Store + 'static>(&mut self, key: String, store: S) {
        self.stores.insert(key, Box::new(store) as Box<Store>);
    }

    fn get_store_mut(&mut self, key: String) -> Option<&mut Box<Store + 'static>> {
        self.stores.get_mut(&key)
    }
}
