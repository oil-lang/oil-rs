
#[macro_export]
macro_rules! declare_data_binding {
    ($name:ident {
        $($field:ident),*
    }) => (
        declare_data_binding! {
            $name {
                $($field -> $field),*
            }
        }
        );
    ($name:ident {
        $($key:ident -> $field:ident),*
    }) => (
        impl $crate::data_bindings::DBStore for $name {

            fn get_value(&self, k: &str) -> Option<$crate::data_bindings::StoreValue> {
                match k {
                    $(stringify!($key) => Some(self.$field.clone().into()),)*
                    _ => None,
                }
            }

            fn set_value(&mut self, k: &str, value: $crate::data_bindings::StoreValue)
                -> Option<$crate::data_bindings::StoreValue>
            {
                match k {
                    $(stringify!($key) => {
                        self.$field = value.into();
                        None
                    })*
                    _ => Some(value),
                }
            }
        }

        impl $crate::data_bindings::BulkGet for $name {

            fn compare_and_update(this: &[$name], k: &str, output: &mut Vec<$crate::data_bindings::StoreValue>)
                -> $crate::data_bindings::BindingResult<bool>
            {
                use $crate::data_bindings::StoreValue;
                use $crate::data_bindings::DataBindingError;
                match k {
                    $(stringify!($key) => {
                        let mut has_changed = false;
                        if output.len() != this.len() {
                            has_changed = true;
                            if output.len() > this.len() {
                                output.truncate(this.len());
                            } else {
                                let len = output.len();
                                output.reserve(this.len() - len);
                            }
                        }
                        for (element, old_value) in this.iter().zip(output.iter_mut()) {
                            let new_value: StoreValue = element.$field.clone().into();
                            if new_value != *old_value {
                                has_changed = true;
                                *old_value = new_value;
                            }
                        }
                        for input in this.iter().skip(output.len()) {
                            (*output).push(input.$field.clone().into());
                        }
                        Ok(has_changed)
                    })*
                    _ => Err(DataBindingError::KeyNotFound(format!(": {} for type {}", k, stringify!($name)))),
                }
            }
        }

        )
}
