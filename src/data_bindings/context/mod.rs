///! DataBinderScope is an implementation detail of DataBinderContext
///!
///! It classify DBStore into categories and act as a unique scope.
///! DataBinderContext build a scope on top of that struct, using
///! the DataBinderScope as the unit for each scope.
///!
///! Note that this concept might be removed in the future
///! to be completely replaced by the Ambient Model concept.
///!


pub use self::binder::DataBinderContext;

mod binder;
mod prefixkey_iter;


use std::collections::HashMap;
use self::prefixkey_iter::PrefixKeyIter;
use data_bindings::DBStore;
use data_bindings::StoreValue;
use data_bindings::IsRepeatable;
use data_bindings::DataBindingError;
use data_bindings::IteratingClosure;
use data_bindings::BindingResult;

#[derive(Default)]
struct DataBinderScope {
    values: HashMap<String,StoreValue>,
    stores: HashMap<String,Box<DBStore>>,
    iterators: HashMap<String,Box<IsRepeatable>>,
}

impl DBStore for DataBinderScope {
    fn get_value(&self, k: &str) -> Option<StoreValue> {
        for (prefix, key) in PrefixKeyIter::new(k) {
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
