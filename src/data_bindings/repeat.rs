use std::rc::{Rc,Weak};
use std::cell::RefCell;
use data_bindings::BindingResult;
use data_bindings::DataBindingError;
use data_bindings::StoreValue;
use data_bindings::IsRepeatable;
use data_bindings::DBStore;
use data_bindings::BulkGet;
use data_bindings::IteratingClosure;

pub struct RepeatProxy<T> {
    cell: Weak<RefCell<Vec<T>>>,
}

impl <T> RepeatProxy<T> {
    pub fn new(cell: &Rc<RefCell<Vec<T>>>) -> RepeatProxy<T> {
        RepeatProxy {
            cell: cell.downgrade(),
        }
    }
}

impl <T> IsRepeatable for RepeatProxy<T>
where T: DBStore + 'static,
      [T]: BulkGet {
    fn iter(&self, closure: &mut IteratingClosure) -> bool {
        let reference = match self.cell.upgrade() {
            Some(r) => r,
            None => return false,
        };
        let mut guard = reference.borrow_mut();
        let mut iter = guard.iter_mut().map(|item| item as &mut DBStore);
        closure(&mut iter);
        true
    }

    fn compare_and_update(&self, k: &str, output: &mut Vec<StoreValue>) -> BindingResult<bool> {
        let reference = match self.cell.upgrade() {
            Some(r) => r,
            None => return Err(DataBindingError::DanglingReference(format!(": {}", k))),
        };
        let guard = reference.borrow_mut();
        (*guard).compare_and_update(k, output)
    }

    fn len(&self) -> BindingResult<u32> {
        let reference = match self.cell.upgrade() {
            Some(r) => r,
            None => return Err(DataBindingError::DanglingReference("".to_string())),
        };
        let guard = reference.borrow();
        Ok((*guard).len() as u32)
    }
}
