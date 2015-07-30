
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
                    _ => $crate::AttributeGetResult::NoSuchProperty,
                }
            }

            fn set_attribute<'a>(&mut self, k: $crate::PropertyAccessor, value: $crate::StoreValue<'a>)
                -> $crate::AttributeSetResult<'a>
            {
                match k.name() {
                    $(stringify!($field) => self.$field.set_attribute(k.next(), value),)*
                    _ => $crate::AttributeSetResult::NoSuchProperty(value),
                }
            }
        }
        )
}

/// To apply this macro to your own type, you need to verify
/// the following trait:
/// ```ignore
/// T where T: AsStoreValue + Cast + Reflect + 'static
/// ```
/// The `impl` couldn't be made generic because of a conflict
/// between this implementation and the generic one for `Box<T>`.
/// TODO(Nemikolh): Was it a compiler bug ?
#[macro_export]
macro_rules! impl_store_for_value_type_like {
    ($type_ident:ident) => (
        impl Store for $type_ident
        {

            fn get_attribute<'b>(&'b self, k: PropertyAccessor) -> AttributeGetResult<'b> {
                match k.name() {
                    "" => {
                        let attribute: StoreValue<'b> = self.as_store_value();
                        AttributeGetResult::PrimitiveType(attribute)
                    }
                    _ => AttributeGetResult::NoSuchProperty
                }
            }

            fn set_attribute<'b>(&mut self, k: PropertyAccessor, value: StoreValue<'b>) -> AttributeSetResult<'b> {
                match k.name() {
                    "" => match <Self as Cast>::cast(value) {
                        Some(c) => { *self = c; AttributeSetResult::Stored },
                        None => AttributeSetResult::WrongType
                    },
                    _ => AttributeSetResult::NoSuchProperty(value)
                }
            }
        }
    )
}
