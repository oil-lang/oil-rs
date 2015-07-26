
#[macro_export]
macro_rules! declare_data_binding {
    ($name:ident {
        $($field:ident),*
    }) => (
        impl $crate::Store for $name {
            fn get_attribute<'a>(&'a self, k: $crate::PropertyAccessor)
                -> $crate::AttributeGetResult<'a>
            {
                match k.name() {
                    $(stringify!($field) => self.$field.get_attribute(k.next()),)*
                    _ => AttributeGetResult::NoSuchProperty,
                }
            }

            fn set_attribute<'a>(&mut self, k: $crate::PropertyAccessor, value: $crate::StoreValue<'a>)
                -> $crate::AttributeSetResult<'a>
            {
                match k.name() {
                    $(stringify!($field) => self.$field.set_attribute(k.next(), value),)*
                    _ => AttributeSetResult::NoSuchProperty(value),
                }
            }
        }
        )
}
