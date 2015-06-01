use std::rc::{Rc,Weak};
use std::cell::RefCell;
use std::collections::HashMap;

use data_bindings::context::DataBinderScope;
use data_bindings::repeat::RepeatProxy;
use data_bindings::Proxy;
use data_bindings::BindingResult;
use data_bindings::StoreValue;
use data_bindings::DataBindingError;
use data_bindings::DBStore;
use data_bindings::BulkGet;
use data_bindings::DBCLookup;
use data_bindings::IteratingClosure;
use router::Router;


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

    pub fn register_global_value(&mut self, key: String, value: StoreValue)
    {
        if let Err(old) = self.global.register_value(key.clone(), value) {
            println!("WARNING: re-registering global value {} (old value {:?})", key, old);
        }
    }

    pub fn register_global_store<T>(&mut self, prefix: String, value: &Rc<RefCell<T>>)
        where T: DBStore + 'static
    {
        let v = Box::new(Proxy::new(value));
        if let Err(_) = self.global.register_store(prefix.clone(), v) {
            println!("WARNING: overriding global object {}", prefix);
        }
    }

    pub fn register_value(&mut self, view: &str, key: String, value: StoreValue) -> BindingResult<()>
    {
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
        where T: DBStore + 'static
    {
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
             [T]: BulkGet
    {
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
             [T]: BulkGet
    {
        let v = Box::new(RepeatProxy::new(iterator));
        if let Err(_) = self.global.register_iterator(key.clone(), v) {
            println!("WARNING: re-registering global iterator {}", key);
        }
    }

    fn register_view(&mut self, view: String) {
        self.views.entry(view).or_insert_with(|| DataBinderScope::default());
    }
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
                Err(DataBindingError::IteratorNotFound(..)) => {} // Normal operation
                Err(e) => return Err(e),
                Ok(out) => return Ok(out),
            }
        }
        self.global.compare_and_update(iterator, k, output)
    }

    fn iterator_len(&self, iterator: &str) -> BindingResult<u32> {
        if let Some(view_scope) = self.views.get(&self.current_view) {
            match view_scope.iterator_len(iterator) {
                Err(DataBindingError::IteratorNotFound(..)) => {} // Normal operation
                Err(e) => return Err(e),
                Ok(out) => return Ok(out),
            }
        }
        self.global.iterator_len(iterator)
    }
}
