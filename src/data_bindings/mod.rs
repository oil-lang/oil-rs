
// Re-export
pub use oil_databindings::{
    Store,
    Cast,
    StoreValue,
    AttributeMutResult,
    AttributeGetResult,
    AttributeSetResult,
    DefaultContextManager,
    DataBindingsContext
};
pub mod context {
    pub use oil_databindings::context::ContextManager;
}

pub use self::buffer::DataBindingBuffer;
mod buffer;

// trait IsRepeatable {
//     fn iter(&self, closure: &mut IteratingClosure) -> bool;
//     fn compare_and_update(&self, k: &str, output: &mut Vec<StoreValue>) -> BindingResult<bool>;
//     fn len(&self) -> BindingResult<u32>;
// }
//
//
// pub trait DBCLookup {
//     fn get_attribute(&self, k: &str) -> Option<StoreValue>;
//     fn set_attribute(&mut self, k: &str, value: StoreValue);
// }
//
//
// impl DBCLookup for ContextManager {
//     fn get_attribute(&self, k: &str) -> Option<StoreValue> {
//         match self.views.get(&self.current_view) {
//             None => {
//                 println!("WARNING: Did not find view {}", &self.current_view);
//             }
//             Some(view_store) => {
//                 let result = view_store.get_attribute(PropertyAccessor::new(k));
//                 if result.is_some() {
//                     return result;
//                 }
//             }
//         }
//         // Did not find view, or view did not have the corresponding value
//         self.global.get_attribute(PropertyAccessor::new(k))
//     }
//
//     fn set_attribute(&mut self, k: &str, value: StoreValue) {
//         match self.views.get_mut(&self.current_view) {
//             None => {
//                 println!("WARNING: Did not find view {}", &self.current_view);
//             }
//             Some(view_store) => {
//                 let result = view_store.set_attribute(PropertyAccessor::new(k), value);
//                 match result {
//                     NoSuchProperty(value) => {
//                         self.global.set_attribute(PropertyAccessor::new(k), value);
//                     },
//                     _ => {}
//                 }
//             }
//         }
//     }
//
//     fn iter(&self, k: &str, closure: &mut IteratingClosure) -> bool {
//         if let Some(view_scope) = self.views.get(&self.current_view) {
//             if view_scope.iter(k, closure) {
//                 return true;
//             }
//         }
//         self.global.iter(k, closure)
//     }
//
//     fn compare_and_update(&self, iterator: &str, k: &str, output: &mut Vec<StoreValue>) -> BindingResult<bool> {
//         if let Some(view_scope) = self.views.get(&self.current_view) {
//             match view_scope.compare_and_update(iterator, k, output) {
//                 Err(DataBindingError::IteratorNotFound(..)) => {} // Normal operation
//                 Err(e) => return Err(e),
//                 Ok(out) => return Ok(out),
//             }
//         }
//         self.global.compare_and_update(iterator, k, output)
//     }
//
//     fn iterator_len(&self, iterator: &str) -> BindingResult<u32> {
//         if let Some(view_scope) = self.views.get(&self.current_view) {
//             match view_scope.iterator_len(iterator) {
//                 Err(DataBindingError::IteratorNotFound(..)) => {} // Normal operation
//                 Err(e) => return Err(e),
//                 Ok(out) => return Ok(out),
//             }
//         }
//         self.global.iterator_len(iterator)
//     }
// }
