use num::ToPrimitive;
use StoreValue;
use store::StoreValueStatic;
use std::ops::Deref;

/// A type that want to be converted into from a `StoreValue`
/// must implement this trait.
pub trait Cast {

    /// Try to cast the given StoreValue into Self.
    /// If the StoreValue has an appropriate type, return
    /// `None`.
    fn cast(this: StoreValue) -> Option<Self>;
}

/// A type to convert a type into a store value
/// but without moving it.
pub trait AsStoreValue {

    /// Should convert the type into a StoreValue.
    /// Note that the lifetime of the StoreValue is bounded
    /// by `self`.
    fn as_store_value(&self) -> StoreValue;
}

/// The friend trait of the `Cast` trait. It is automatically implemented
/// and shouldn't be implemented directly. Rely on the `Cast` trait instead.
pub trait AssignFromCast {

    fn assign(&mut self, this: StoreValue);
}

impl<T> AssignFromCast for T
    where T: Cast
{
    fn assign(&mut self, this: StoreValue) {
        match <Self as Cast>::cast(this) {
            Some(v) => *self = v,
            None => (),
        }
    }
}

impl Cast for StoreValueStatic {
    fn cast(this: StoreValue) -> Option<Self> {
        match this {
            StoreValue::String(s) => Some(StoreValueStatic::String(s.to_string())),
            StoreValue::Integer(i) => Some(StoreValueStatic::Integer(i)),
            StoreValue::Boolean(b) => Some(StoreValueStatic::Boolean(b)),
        }
    }
}

impl AsStoreValue for StoreValueStatic {
    fn as_store_value(&self) -> StoreValue {
        match self {
            &StoreValueStatic::String(ref s) => StoreValue::String(s.deref()),
            &StoreValueStatic::Integer(i) => StoreValue::Integer(i),
            &StoreValueStatic::Boolean(b) => StoreValue::Boolean(b),
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
        impl AsStoreValue for $type_ident {
            fn as_store_value(&self) -> StoreValue {
                StoreValue::Integer(*self as i64)
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
                Some(s.to_string())
            }
            StoreValue::Integer(i) => {
                Some(i.to_string())
            }
            StoreValue::Boolean(b) => {
                Some(b.to_string())
            }
        }
    }
}

impl AsStoreValue for String {
    fn as_store_value(&self) -> StoreValue {
        StoreValue::String(self.deref())
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
        }
    }
}

impl AsStoreValue for bool {
    fn as_store_value(&self) -> StoreValue {
        StoreValue::Boolean(*self)
    }
}
