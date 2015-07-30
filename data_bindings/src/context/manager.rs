use std::collections::HashMap;

use {
    StoreValue,
    Store,
    PropertyAccessor,
    AttributeSetResult,
    AttributeGetResult,
};
use store::StoreValueStatic;
use super::Context;

/// The `ContextManager` is templated by a class implementing
/// the `Store` trait. It will then be your only way to store
/// values that can be bind and used in views.
#[derive(Default)]
pub struct ContextManager<G: Default, V> {
    global: G,
    views: HashMap<String, V>,
}

impl<G, V> ContextManager<G, V>
    where G: Store + Default,
          V: Store
{

    /// Create a new context manager using the specified model.
    ///
    pub fn new(store: G) -> ContextManager<G, V> {
        ContextManager {
            global: store,
            views: HashMap::new()
        }
    }

    /// Insert a root `Store` for the given view.
    /// **Note:**
    ///     With the `ContextManager`, all views have a store of the same type.
    ///     But if this is not what you want, you can implement
    ///     the trait `...` TODO(Nemikolh)
    pub fn insert_view_level_store(&mut self, view_name: String, store: V) {
        self.views.insert(view_name, store);
    }

    /// Equivalent to the get_attribute of the `Store` trait.
    pub fn get_attribute<'a>(&'a mut self, view: &str, key: &str) -> Option<StoreValue<'a>> {
        match self.views.get(view) {
            Some(store) =>
                if let AttributeGetResult::PrimitiveType(sv) = store
                    .get_attribute(PropertyAccessor::new(key)) {
                    return Some(sv);
                },
            _ => ()
        }
        if let AttributeGetResult::PrimitiveType(sv) = self.global
            .get_attribute(PropertyAccessor::new(key)) {
            Some(sv)
        } else {
            None
        }
    }

    /// Equivalent to the `set_attribute` of the `Store` trait.
    /// TODO(Nemikolh): Move that comment for the trait `...`
    /// The view argument can be ignored. It offers the possibility
    /// To have different root stores per view.
    pub fn set_attribute<'a>(&mut self, view: &str, key: &str, value: StoreValue<'a>) {
        if let AttributeSetResult::NoSuchProperty(sv) = self.views.get_mut(view)
            .unwrap()
            .set_attribute(PropertyAccessor::new(key), value) {
            if let AttributeSetResult::NoSuchProperty(_) = self.global
                .set_attribute(PropertyAccessor::new(key), sv) {
                // Insertion failed.
            }
        }
    }
}

/// Implement `Context` equivalent method per view,
/// if the type `V` implement `Context`.
impl<G, V> ContextManager<G, V>
    where V: Context,
          G: Default
{
    pub fn register_store_for_view<S: Store>(
        &mut self,
        view_name: String,
        store_name: String,
        store: S) {
        self.views.get_mut(&view_name).unwrap().register_store(store_name, store);
    }

    pub fn register_value_for_view<M: Into<StoreValueStatic>>(
        &mut self,
        view_name: String,
        store_name: String,
        value: M) {
        self.views.get_mut(&view_name).unwrap().register_value(store_name, value);
    }
}

/// Implement `Context` equivalent method for the global data,
/// if the type `G` implement `Context`.
impl<G, V> ContextManager<G, V>
    where G: Context + Default
{
    pub fn register_global_store<S: Store>(
        &mut self,
        store_name: String,
        store: S)
    {
        self.global.register_store(store_name, store);
    }


    pub fn register_global_value<M: Into<StoreValueStatic>>(
        &mut self,
        store_name: String,
        value: M)
    {
        self.global.register_value(store_name, value);
    }
}


// ======================================== //
//                   TESTS                  //
// ======================================== //

#[cfg(test)]
mod test {
}
