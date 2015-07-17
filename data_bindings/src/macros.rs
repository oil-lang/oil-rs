
#[macro_export]
macro_rules! declare_data_binding {
    ($name:ident {
        $($field:ident),*
    }) => (
        impl $crate::data_bindings::Store for $name {
            fn get_attribute(&self, k: $crate::data_bindings::PropertyAccessor)
                -> $crate::data_bindings::AttributeGetSingleResult
            {
                match k.name() {
                    $(Some(stringify!($field)) => self.$field.get_attribute(k.next()),)*
                    _ => AttributeGetSingleResult::NoSuchProperty,
                }
            }

            fn set_attribute(&mut self, k: $crate::data_bindings::PropertyAccessor, value: $crate::data_bindings::StoreValue)
                -> $crate::data_bindings::AttributeSetResult
            {
                match k.name() {
                    $(stringify!($field) => self.$field.set_attribute(k.next()),)*
                    _ => AttributeSetResult::NoSuchProperty(value),
                }
            }
        }
        )
}
