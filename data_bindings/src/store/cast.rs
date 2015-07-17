use num::ToPrimitive;
use StoreValue;
use store::StoreValueStatic;

/// A type that want to be converted into from a `StoreValue`
/// must implement this trait.
pub trait Cast {
    
    /// Try to cast the given StoreValue into Self.
    /// If the StoreValue has an appropriate type, return
    /// `None`.
    fn cast(this: StoreValue) -> Option<Self>;
}

impl Cast for StoreValueStatic {
    
    fn cast(this: StoreValue) -> Option<Self> {
        match this {
            StoreValue::String(s) => Some(StoreValueStatic(StoreValue::String(s))),
            StoreValue::Integer(i) => Some(StoreValueStatic(StoreValue::Integer(i))),
            StoreValue::Boolean(b) => Some(StoreValueStatic(StoreValue::Boolean(b))),
            StoreValue::List(_) => None
        }
    }
}

/// Implementation for integers
macro_rules! impl_for_integer {
    ($type_ident:ident, $to_value:ident) => (
        impl Cast for $type_ident {
            fn cast(this: StoreValue) -> Option<Self> {
                match this {
                    StoreValue::Integer(i) => {
                        i.$to_value()
                    }
                    _ => None
                }
            }
        }
    )
}

impl_for_integer!(i64, to_i64);
impl_for_integer!(i32, to_i32);
impl_for_integer!(i16, to_i16);
impl_for_integer!(i8,  to_i8);

impl_for_integer!(u32, to_u32);
impl_for_integer!(u16, to_u16);
impl_for_integer!(u8,  to_u8);

impl Cast for String {
    fn cast(this: StoreValue) -> Option<Self> {
        match this {
            StoreValue::String(s) => {
                Some(s)
            }
            StoreValue::Integer(i) => {
                Some(i.to_string())
            }
            StoreValue::Boolean(b) => {
                Some(b.to_string())
            }
            _ => None,
        }
    }
}

impl Cast for bool {
    fn cast(this: StoreValue) -> Option<Self> {
        match this {
            StoreValue::Boolean(b) => {
                Some(b)
            },
            StoreValue::String(s) => {
                Some(s.is_empty())
            },
            StoreValue::Integer(i) => {
                Some(i != 0)
            }
            _ => None,
        }
    }
}