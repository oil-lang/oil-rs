use std::rc::{Rc,Weak};
use std::collections::hash_map::{HashMap,Entry};
use std::collections::hash_state::HashState;
use std::cell::RefCell;

use router::Router;

#[derive(Clone,Debug)]
pub struct StoreValue {
    last_changed: usize,
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
pub struct DataBinderContext {
    global_values: HashMap<String,StoreValue>,
    global_context: Vec<Box<DBStore>>,
    views_context: HashMap<String,Vec<Box<DBStore>>>,
    current_view: String,
}

impl DataBinderContext {
    pub fn new(router: &Router) -> DataBinderContext {
        let mut binder = DataBinderContext::default();
        binder.register_views(router);
        binder
    }

    pub fn register_views(&mut self, router: &Router) {
        // XXX: Could have better implementation: use Entry
        for name in router.iter_name_views() {
            self.views_context.entry(name.clone()).or_insert(Vec::new());
        }
    }

    pub fn register_global_single_value(&mut self, key: String, value: StoreValue) {
        // XXX: Any better way to handle the log?
        match self.global_values.entry(key.clone()) {
            Entry::Occupied(mut occupied) => {
                // TODO: Use log
                println!("WARNING: registering an already existing value for key {} : {:?}", key, occupied.get());
                occupied.insert(value);
            }
            Entry::Vacant(v) => {
                v.insert(value);
            }
        }
    }

    pub fn register_global<T>(&mut self, scope: usize, value: Rc<RefCell<T>>)
    where T: DBStore + 'static {
        let v = Box::new(Proxy::new(value));
        let len = self.global_context.len();
        self.global_context.insert(::std::cmp::max(scope, len), v);
    }

    pub fn register_for_view<T>(&mut self, view: &str, scope: usize, value: Rc<RefCell<T>>) -> Result<(),String>
    where T: DBStore + 'static {
        let v = Box::new(Proxy::new(value));
        let len = self.global_context.len();
        match self.views_context.get_mut(view) {
            None => Err(format!("Could not find view {}", view)),
            Some(vec) => {
                vec.insert(::std::cmp::max(scope, len), v);
                Ok(())
            }
        }
    }
}

// Private trait for UIL
pub trait DBCLookup {
    fn get_value(&self, k: &str) -> Option<StoreValue>;
    fn set_value(&mut self, k: &str, value: StoreValue);
}

impl DBCLookup for DataBinderContext {
    fn get_value(&self, k: &str) -> Option<StoreValue> {
        match self.views_context.get(&self.current_view) {
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
        let result = self.global_context.get_value(k);
        if result.is_some() {
            return result;
        }
        self.global_values.get_value(k)
    }

    fn set_value(&mut self, k: &str, value: StoreValue) {
        match self.views_context.get_mut(&self.current_view) {
            None => {
                println!("WARNING: Did not find view {}", &self.current_view);
            }
            Some(view_store) => {
                let result = view_store.set_value(k, value);
                match result {
                    None => {},
                    Some(a) => {
                        match self.global_context.set_value(k, a) {
                            None => {},
                            Some(a) => {self.global_values.set_value(k, a);}
                        }
                    }
                }
            }
        }
    }
}

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
