use std::rc::{Rc,Weak};
use std::cell::RefCell;
use std::ops::Deref;
use data_bindings::{
    BindingResult,
    DataBindingError,
    StoreValue,
    IsRepeatable,
    Store,
    ArrStore,
    IteratingClosure,
    PropertyAccessor,
    AttributeSetResult
};


#[derive(Debug)]
pub struct Proxy<T: Store> {
    data: Weak<RefCell<T>>,
}

impl <T> Store for Proxy<T>
where T: Store {
    fn get_attribute(&self, k: PropertyAccessor) -> AttributeGetResult {
        match self.data.upgrade() {
            None => NoSuchProperty,
            Some(p) => p.borrow().get_attribute(k),
        }
    }

    fn set_attribute(&mut self, k: PropertyAccessor, value: StoreValue) -> AttributeSetResult {
        match self.data.upgrade() {
            None => NoSuchProperty(value),
            Some(p) => p.borrow_mut().set_attribute(k, value),
        }
    }
}

impl <T: Store> Proxy<T> {
    pub fn new(value: &Rc<RefCell<T>>) -> Proxy<T> {
        Proxy {
            data: value.downgrade(),
        }
    }
}


pub struct RepeatProxy<T> {
    weak_values: Weak<RefCell<Vec<T>>>,
}

impl <T> RepeatProxy<T> {
    pub fn new(values: &Rc<RefCell<Vec<T>>>) -> RepeatProxy<T> {
        RepeatProxy {
            weak_values: values.downgrade(),
        }
    }
}

impl <T> IsRepeatable for RepeatProxy<T>
where T: Store + ArrStore + 'static {

    fn iter(&self, closure: &mut IteratingClosure) -> bool {
        let ref_values = match self.weak_values.upgrade() {
            Some(r) => r,
            None => return false,
        };
        let mut values = ref_values.borrow_mut();
        let mut iter = values.iter_mut().map(|item| item as &mut Store);
        closure(&mut iter);
        true
    }

    fn compare_and_update(&self, k: &str, output: &mut Vec<StoreValue>) -> BindingResult<bool> {
        let ref_values = match self.weak_values.upgrade() {
            Some(r) => r,
            None => return Err(DataBindingError::DanglingReference(format!(": {}", k))),
        };
        let values = ref_values.borrow_mut();
        ArrStore::compare_and_update((*values).deref(), k, output)
    }

    fn len(&self) -> BindingResult<u32> {
        let ref_values = match self.weak_values.upgrade() {
            Some(r) => r,
            None => return Err(DataBindingError::DanglingReference("".to_string())),
        };
        let values = ref_values.borrow();
        Ok((*values).len() as u32)
    }
}
