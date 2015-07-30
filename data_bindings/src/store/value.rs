/// `StoreValue` is the type that encapsulate
/// a value extracted from a Store
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum StoreValue<'a> {
    String(&'a str),
    Integer(i64),
    Boolean(bool),
}

/// Equivalent to of `StoreValue<'static>` to allow the implementation
/// of a :
/// ```ignore
/// impl<'a> From<StoreValue<'static>> for StoreValue<'a> {
///     ...
/// }
/// ```
#[derive(Clone)]
pub enum StoreValueStatic {
    String(String),
    Integer(i64),
    Boolean(bool),
}

macro_rules! impl_for_integer {
    ($int_type:ident) => (
        impl From<$int_type> for StoreValueStatic {
            fn from(i: $int_type) -> StoreValueStatic {
                StoreValueStatic::Integer(i as i64)
            }
        }
    )
}

impl_for_integer!(i64);
impl_for_integer!(i32);
impl_for_integer!(i16);
impl_for_integer!(i8);

impl_for_integer!(u32);
impl_for_integer!(u16);
impl_for_integer!(u8);

impl From<String> for StoreValueStatic {
    fn from(s: String) -> StoreValueStatic {
        StoreValueStatic::String(s)
    }
}

impl From<bool> for StoreValueStatic {
    fn from(b: bool) -> StoreValueStatic {
        StoreValueStatic::Boolean(b)
    }
}
