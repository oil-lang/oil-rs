
#[macro_export]
macro_rules! declare_data_binding {
    ($name:ident {
        $($field:ident:$type_field:ty),*
    }) => (
        declare_data_binding! {
            $name {
                $($field -> $field: $type_field),*
            }
        }
        );
    ($name:ident {
        $($key:ident -> $field:ident : $type_field:ty),*
    }) => (
        impl DBStore for $name {
            fn get_value(&self, k: &str) -> Option<StoreValue> {
                match k {
                    $(stringify!($key) => Some(StoreValue::from(self.$field.clone())),)*
                    _ => None,
                }
            }
            fn set_value(&mut self, k: &str, value: StoreValue) -> Option<StoreValue> {
                match k {
                    $(stringify!($key) => {
                        self.$field = value.into();
                        None
                    })*
                    _ => Some(value),
                }
            }
        }

        impl BulkGet for [$name] {
            fn compare_and_update(&self, k: &str, output: &mut Vec<StoreValue>) -> BindingResult<bool> {
                match k {
                    $(stringify!($key) => {
                        let mut has_changed = false;
                        if output.len() != self.len() {
                            has_changed = true;
                            if output.len() > self.len() {
                                output.truncate(self.len());
                            } else {
                                let len = output.len();
                                output.reserve(self.len() - len);
                            }
                        }
                        for (element, old_value) in self.iter().zip(output.iter_mut()) {
                            let new_value: StoreValue = element.$field.clone().into();
                            if new_value != *old_value {
                                has_changed = true;
                                *old_value = new_value;
                            }
                        }
                        for input in self.iter().skip(output.len()) {
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
