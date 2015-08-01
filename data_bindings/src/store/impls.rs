use std::collections::{
    HashMap
};
use std::slice;
use std::marker::Reflect;
use std::collections::hash_state::HashState;

use lookup::{
    PropertyAccessor,
    PrefixKeyIter
};
use store::StoreValueStatic;
use store::cast::AsStoreValue;
use {
    AttributeSetResult,
    AttributeGetResult,
    AttributeMutResult,
    Store,
    StoreValue,
};

/// `Box` implementation to allow virtual implementations
impl<T: ?Sized> Store for Box<T> where T: Store {
    fn get_attribute(&self, k: PropertyAccessor) -> AttributeGetResult {
        (**self).get_attribute(k)
    }
    fn get_attribute_mut(&mut self, k: PropertyAccessor) -> AttributeMutResult {
        (**self).get_attribute_mut(k)
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
impl_store_for_value_type_like!(i64);
impl_store_for_value_type_like!(i32);
impl_store_for_value_type_like!(i16);
impl_store_for_value_type_like!(i8);
impl_store_for_value_type_like!(u32);
impl_store_for_value_type_like!(u16);
impl_store_for_value_type_like!(u8);
impl_store_for_value_type_like!(bool);
impl_store_for_value_type_like!(String);
impl_store_for_value_type_like!(StoreValueStatic);

struct WrapperIter<'a, T: 'a> {
    it: slice::Iter<'a, T>,
}

struct WrapperIterMut<'a, T: 'a> {
    it: slice::IterMut<'a, T>,
}

impl<'a, T> Iterator for WrapperIter<'a, T>
    where T: Store + 'a
{
    type Item = &'a Store;

    fn next(&mut self) -> Option<Self::Item> {
        self.it.next().map(|i| i as &Store)
    }
}

impl<'a, T> Iterator for WrapperIterMut<'a, T>
    where T: Store + 'a
{
    type Item = &'a mut Store;

    fn next(&mut self) -> Option<Self::Item> {
        self.it.next().map(|i| i as &mut Store)
    }
}
/*impl<'a, T> From<&'a [T]> for StoreValue<'a>
    where T: Store
{

    fn from(t: &'a [T]) -> StoreValue<'a> {
        let i = Box::new(WrapperIter { it: t.iter() }) as Box<Iterator<Item=&'a Store> + 'a>;
        StoreValue::List(i)
    }
}
*/


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
            "" => AttributeGetResult::IterableType(
                Box::new(WrapperIter { it: self.iter() }) as Box<Iterator<Item=&'a Store> + 'a>
            ),
            _ => AttributeGetResult::NoSuchProperty
        }
    }

    fn get_attribute_mut<'a>(&'a mut self, k: PropertyAccessor) -> AttributeMutResult<'a> {
        match k.name() {
            "" => AttributeMutResult::IterableType(
                Box::new(WrapperIterMut { it: self.iter_mut() }) as Box<Iterator<Item=&'a mut Store> + 'a>
            ),
            _ => AttributeMutResult::NoSuchProperty
        }
    }

    // TODO(Nemikolh): We should be able to modify the element of the array.
    // But in order to have a nicer interface it would be better to have something like:
    //
    //      get_attribute_mut() -> AttributeMutResult
    //
    // where:
    //
    //      AttributeMutResult<'a> {
    //          PrimitiveType(&'a mut Cast),
    //          IterableType()
    //      }
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
                    default_case => return default_case,
                }
            }
        }
        AttributeGetResult::NoSuchProperty
    }

    fn get_attribute_mut<'a>(&'a mut self, k: PropertyAccessor) -> AttributeMutResult<'a> {
        // TODO(Nemikolh): Strangely this code doesn't compile.
        // for (prefix, key) in PrefixKeyIter::new(k) {
        //     if let Some(store) = self.get_mut(prefix) {
        //         match store.get_attribute_mut(key) {
        //             AttributeMutResult::NoSuchProperty => continue,
        //             default_case => return default_case,
        //         }
        //     }
        // }
        //
        // TODO(Nemikolh): Slow hack to get the job done.
        let mut kfound = None;
        let mut pfound = None;
        for (prefix, key) in PrefixKeyIter::new(k) {
            kfound = Some(key.clone());
            if let Some(store) = self.get(prefix) {
                match store.get_attribute(key) {
                    AttributeGetResult::NoSuchProperty => (),
                    _ => {
                        pfound = Some(prefix);
                        break;
                    }
                }
            }
        }
        if pfound.is_some() && kfound.is_some() {
            let store = self.get_mut(pfound.unwrap()).unwrap();
            store.get_attribute_mut(kfound.unwrap())
        } else {
            AttributeMutResult::NoSuchProperty
        }
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

// ======================================== //
//                   TESTS                  //
// ======================================== //

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

        let mut iter = v.get_attribute(PropertyAccessor::new("a")).unwrap_iter();
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
            let iter = v.get_attribute(PropertyAccessor::new("a")).unwrap_iter();
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
