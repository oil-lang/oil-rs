use std::collections::{
    HashMap
};
use std::ops::Deref;
use std::marker::Reflect;
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
impl Store for Box<Store + 'static> {
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
    where T: Into<StoreValueStatic> + Clone + Cast + Reflect + 'static
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
impl<T> Store for Vec<T>
    where T: Store + Reflect
{
    fn get_attribute<'a>(&'a self, k: PropertyAccessor) -> AttributeGetResult<'a> {
        match k.name() {
            "" => AttributeGetResult::Found(self.deref().into()),
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
    where S: HashState + Reflect + 'static,
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

#[cfg(test)]
mod test {

    use store::StoreValue;
    use Store;
    use PropertyAccessor;
    use bench::Bencher;
    use num::traits::ToPrimitive;

    #[derive(Clone)]
    struct A {
        a: Vec<B>,
    }

    declare_data_binding! {
        A {
            a
        }
    }

    #[derive(Clone)]
    struct B {
        b: u32,
        c: C,
    }

    declare_data_binding! {
        B {
            b,
            c
        }
    }

    #[derive(Clone)]
    struct C {
        a: Vec<A>,
    }

    declare_data_binding! {
        C {
            a
        }
    }

    #[test]
    fn vec_access_should_return_box_of_elements() {
        let v = A {
            a: vec![B { b: 0, c: C { a: Vec::new() }},
                    B { b: 1, c: C { a: Vec::new() }}
            ]
        };

        let mut iter = match v.get_attribute(PropertyAccessor::new("a")).unwrap() {
            StoreValue::List(iter) => iter,
            _ => panic!("test failed on iter access"),
        };
        assert_eq!(iter.next().unwrap().get_attribute(PropertyAccessor::new("b")).unwrap(), StoreValue::Integer(0));
        assert_eq!(iter.next().unwrap().get_attribute(PropertyAccessor::new("b")).unwrap(), StoreValue::Integer(1));
    }

    #[bench]
    fn vec_direct_access(b: &mut Bencher) {
        let total = 1000;
        let v = A {
            a: vec![B { b: 1, c: C { a: Vec::new() }}; total],
        };

        b.iter(|| {
            let mut sum = 0;
            for i in v.a.iter() {
                sum += i.b;
            }
            assert_eq!(sum, total.to_u32().unwrap());
        });
    }

    #[bench]
    fn vec_store_access(b: &mut Bencher) {
        let total = 1000;
        let v = A {
            a: vec![B { b: 1, c: C { a: Vec::new() }}; total],
        };

        b.iter(|| {
            let iter = match v.get_attribute(PropertyAccessor::new("a")).unwrap() {
                StoreValue::List(iter) => iter,
                _ => panic!("test failed on iter access"),
            };
            let mut sum = 0;
            for i in iter {
                sum += match i.get_attribute(PropertyAccessor::new("b")).unwrap() {
                    StoreValue::Integer(i) => i,
                    _ => 0,
                }
            }
            assert_eq!(sum, total.to_i64().unwrap());
        });
    }
}
