///! AmbientModel is an implementation detail of ContextManager
///!
///! It classify DBStore into categories and act as a unique scope.
///! ContextManager build a scope on top of that struct, using
///! the AmbientModel as the unit for each scope.
///!
///! Note that this concept might be removed in the future
///! to be completely replaced by the Ambient Model concept.
///!


pub use self::manager::ContextManager;

mod manager;
mod prefixkey_iter;
mod proxies;

use std::collections::HashMap;
use self::prefixkey_iter::PrefixKeyIter;
use data_bindings::{
    DBStore,
    StoreValue,
    IsRepeatable,
    DataBindingError,
    IteratingClosure,
    BindingResult,
    PropertyAccessor
};

#[derive(Default)]
struct AmbientModel {
    values: HashMap<String,StoreValue>,
    stores: HashMap<String,Box<DBStore>>,
    iterators: HashMap<String,Box<IsRepeatable>>,
}

impl DBStore for AmbientModel {
    fn get_attribute(&self, k: PropertyAccessor) -> Option<StoreValue> {
        if let Some(name) = k.name() {
            if let Some(store) = self.stores.get(name) {
                return store.get_attribute(k.next())
            }
        }
        self.values.get_attribute(k)
        // for (prefix, key) in PrefixKeyIter::new(k) {
        //     if let Some(store) = self.stores.get(prefix) {
        //         // If we have a store registered, we look here first
        //         let result = store.get_attribute(key);
        //         if result.is_some() {
        //             return result;
        //         }
        //     }
        // }
    }

    fn set_value(&mut self, k: &str, mut value: StoreValue) -> Option<StoreValue> {
        for (prefix, key) in PrefixKeyIter::new(k) {
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

impl AmbientModel {
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
