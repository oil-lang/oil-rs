use std::collections::{
    HashMap
};
use std::collections::hash_state::HashState;

use lookup::{
    PropertyAccessor,
    PrefixKeyIter
};
use store::StoreValueStatic;
use {
    AttributeSetResult,
    AttributeGetResult,
    Store,
    StoreValue,
    Cast
};

/// `Box` implementation to allow virtual implementations
impl<'a> Store for Box<Store + 'a> {
    fn get_attribute<'b>(&'b self, k: PropertyAccessor) -> AttributeGetResult<'b> {
        (**self).get_attribute(k)
    }
    fn set_attribute<'b>(&mut self, k: PropertyAccessor, value: StoreValue<'b>) -> AttributeSetResult<'b> {
        (**self).set_attribute(k, value)
    }
}

/// `Box` implementation to allow virtual implementations
impl<T> Store for Box<T> where T: Store {
    fn get_attribute<'b>(&'b self, k: PropertyAccessor) -> AttributeGetResult<'b> {
        (**self).get_attribute(k)
    }
    fn set_attribute<'b>(&mut self, k: PropertyAccessor, value: StoreValue<'b>) -> AttributeSetResult<'b> {
        (**self).set_attribute(k, value)
    }
}
/*
impl<T> Store for Rc<RefCell<T>> where T: Store {
    fn get_attribute<'b>(&'b self, k: PropertyAccessor) -> AttributeGetResult<'b> {
        let b = self.borrow();
        let a = b.get_attribute(k);
        match 
    }
    fn set_attribute<'b>(&mut self, k: PropertyAccessor, value: StoreValue<'b>) -> AttributeSetResult<'b> {
        self.borrow_mut().set_attribute(k, value)
    }
}
impl<T> Store for Rc<T> where T: Store {
    fn get_attribute<'b>(&'b self, k: PropertyAccessor) -> AttributeGetResult<'b> {
        (**self).get_attribute(k)
    }
    fn set_attribute<'b>(&mut self, k: PropertyAccessor, value: StoreValue<'b>) -> AttributeSetResult<'b> {
        self.set_attribute(k, value)
    }
}
*/

// Implementation for i64, String and others.
impl<T> Store for T
    where T: Into<StoreValueStatic> + Clone + Cast
{

    fn get_attribute<'b>(&'b self, k: PropertyAccessor) -> AttributeGetResult<'b> {
        match k.name() {
            "" => {
                let attribute: StoreValue<'b> = self.clone().into().into();
                AttributeGetResult::Found(attribute)
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

/// This implementation is used by the `repeat`
/// tag. It doesn't allow for set_attribute to do any change
/// to the array. (No tag actually allow that)
/// It works similarly to the impl above for `T` when `T: Into<StoreValue> + Cast`.
/// The get_attribute transforms `&'a [T]` into a `StoreValue::List`
impl<T> Store for [T]
    where T: Store
{
    fn get_attribute<'a>(&'a self, k: PropertyAccessor) -> AttributeGetResult<'a> {
        match k.name() {
            "" => AttributeGetResult::Found(self.into()),
            _ => AttributeGetResult::NoSuchProperty
        }
    }
    
    fn set_attribute<'a>(&mut self, k: PropertyAccessor, value: StoreValue<'a>) -> AttributeSetResult<'a> {
        match k.name() {
            "" => AttributeSetResult::WrongType,
            _ => AttributeSetResult::NoSuchProperty(value)
        }
    }
}

/// This implementation allows for property names such as `foo.bar`
/// The rule follows the logic given by the `PrefixKeyIter` iterator. 
impl<S, T> Store for HashMap<String, T, S>
    where S: HashState + 'static,
          T: Store
{

    fn get_attribute<'a>(&'a self, k: PropertyAccessor) -> AttributeGetResult<'a> {
        for (prefix, key) in PrefixKeyIter::new(k) {
            if let Some(store) = self.get(prefix) {
                match store.get_attribute(key) {
                    AttributeGetResult::NoSuchProperty => (),
                    AttributeGetResult::Found(v) => return AttributeGetResult::Found(v),
                }
            }
        }
        AttributeGetResult::NoSuchProperty
    }

    fn set_attribute<'a>(&mut self, k: PropertyAccessor, mut value: StoreValue<'a>) -> AttributeSetResult<'a> {
        for (prefix, key) in PrefixKeyIter::new(k) {
            if let Some(store) = self.get_mut(prefix) {
                // If we have a store registered, we look here first
                match store.set_attribute(key, value) {
                    AttributeSetResult::NoSuchProperty(v) => value = v,
                    AttributeSetResult::WrongType => return AttributeSetResult::WrongType,
                    AttributeSetResult::Stored => return AttributeSetResult::Stored,
                }
            }
        }
        AttributeSetResult::NoSuchProperty(value)
    }
}
