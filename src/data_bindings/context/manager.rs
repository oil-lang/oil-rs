use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use data_bindings::context::AmbientModel;
use data_bindings::context::proxies::Proxy;
use data_bindings::context::proxies::RepeatProxy;
use data_bindings::{
    BindingResult,
    StoreValue,
    DataBindingError,
    DBStore,
    BulkGet,
    DBCLookup,
    IteratingClosure,
    PropertyAccessor
};
use router::Router;


#[derive(Default)]
pub struct ContextManager {
    global: AmbientModel,
    views: HashMap<String,AmbientModel>,
    current_view: String,
}

impl ContextManager {

    /// Create a new context manager using a router.
    ///
    pub fn new(router: &Router) -> ContextManager {
        let mut binder = ContextManager::default();
        binder.register_views(router);
        binder
    }

    pub fn register_views(&mut self, router: &Router) {
        for name in router.iter_name_views() {
            self.register_view(name.clone());
        }
    }

    pub fn switch_to_view(&mut self, view: String) {
        self.current_view = view;
    }

    pub fn register_global_value<V: Into<StoreValue>>(&mut self, key: String, value: V)
    {
        if let Err(old) = self.global.register_value(key.clone(), value.into()) {
            println!("WARNING: re-registering global value {} (old value {:?})", key, old);
        }
    }

    pub fn register_global_store<T>(&mut self, prefix: String, value: &Rc<RefCell<T>>)
        where T: DBStore + 'static
    {
        let v = Box::new(Proxy::new(value));
        if let Err(_) = self.global.register_store(prefix.clone(), v) {
            println!("WARNING: overriding global object {}", prefix);
        }
    }

    pub fn register_value<V: Into<StoreValue>>(&mut self, view: &str, key: String, value: V) -> BindingResult<()>
    {
        match self.views.get_mut(view) {
            None => Err(DataBindingError::ViewNotFound),
            Some(view_scope) => {
                if let Err(old) = view_scope.register_value(key.clone(), value.into()) {
                    // Don't throw an error, just print a warning
                    println!("WARNING: View {}: re-registering value {} (old value {:?})", view, key, old);
                }
                Ok(())
            }
        }
    }

    pub fn register_store<T>(&mut self, view: &str, prefix: String, value: &Rc<RefCell<T>>) -> BindingResult<()>
        where T: DBStore + 'static
    {
        match self.views.get_mut(view) {
            None => Err(DataBindingError::ViewNotFound),
            Some(view_scope) => {
                let v = Box::new(Proxy::new(value));
                if let Err(_) = view_scope.register_store(prefix.clone(), v) {
                    // Don't throw an error, just print a warning
                    println!("WARNING: View {}: overriding object {}", view, prefix);
                }
                Ok(())
            }
        }
    }

    pub fn register_iterator<T>(&mut self, view: &str, key: String, iterator: &Rc<RefCell<Vec<T>>>) -> BindingResult<()>
        where T: DBStore + BulkGet + 'static
    {
        match self.views.get_mut(view) {
            None => Err(DataBindingError::ViewNotFound),
            Some(view_scope) => {
                let v = Box::new(RepeatProxy::new(iterator));
                if let Err(_) = view_scope.register_iterator(key.clone(), v) {
                    // Don't throw an error, just print a warning
                    println!("WARNING: View {}: overriding iterator {}", view, key);
                }
                Ok(())
            }
        }
    }

    pub fn register_global_iterator<T>(&mut self, key: String, iterator: &Rc<RefCell<Vec<T>>>)
        where T: DBStore + BulkGet + 'static
    {
        let v = Box::new(RepeatProxy::new(iterator));
        if let Err(_) = self.global.register_iterator(key.clone(), v) {
            println!("WARNING: re-registering global iterator {}", key);
        }
    }

    fn register_view(&mut self, view: String) {
        self.views.entry(view).or_insert_with(|| AmbientModel::default());
    }
}


impl DBCLookup for ContextManager {
    fn get_attribute(&self, k: &str) -> Option<StoreValue> {
        match self.views.get(&self.current_view) {
            None => {
                println!("WARNING: Did not find view {}", &self.current_view);
            }
            Some(view_store) => {
                let result = view_store.get_attribute(PropertyAccessor::new(k));
                if result.is_some() {
                    return result;
                }
            }
        }
        // Did not find view, or view did not have the corresponding value
        self.global.get_attribute(PropertyAccessor::new(k))
    }

    fn set_value(&mut self, k: &str, value: StoreValue) {
        match self.views.get_mut(&self.current_view) {
            None => {
                println!("WARNING: Did not find view {}", &self.current_view);
            }
            Some(view_store) => {
                let result = view_store.set_value(k, value);
                match result {
                    None => {},
                    Some(value) => {
                        self.global.set_value(k, value);
                    }
                }
            }
        }
    }

    fn iter(&self, k: &str, closure: &mut IteratingClosure) -> bool {
        if let Some(view_scope) = self.views.get(&self.current_view) {
            if view_scope.iter(k, closure) {
                return true;
            }
        }
        self.global.iter(k, closure)
    }

    fn compare_and_update(&self, iterator: &str, k: &str, output: &mut Vec<StoreValue>) -> BindingResult<bool> {
        if let Some(view_scope) = self.views.get(&self.current_view) {
            match view_scope.compare_and_update(iterator, k, output) {
                Err(DataBindingError::IteratorNotFound(..)) => {} // Normal operation
                Err(e) => return Err(e),
                Ok(out) => return Ok(out),
            }
        }
        self.global.compare_and_update(iterator, k, output)
    }

    fn iterator_len(&self, iterator: &str) -> BindingResult<u32> {
        if let Some(view_scope) = self.views.get(&self.current_view) {
            match view_scope.iterator_len(iterator) {
                Err(DataBindingError::IteratorNotFound(..)) => {} // Normal operation
                Err(e) => return Err(e),
                Ok(out) => return Ok(out),
            }
        }
        self.global.iterator_len(iterator)
    }
}


// ======================================== //
//                   TESTS                  //
// ======================================== //

#[cfg(test)]
mod test {
    use std::rc::Rc;
    use std::cell::RefCell;
    use data_bindings::StoreValue;
    use data_bindings::DataBindingError;
    use data_bindings::BindingResult;
    use data_bindings::DBStore;
    use data_bindings::BulkGet;
    use data_bindings::DBCLookup;
    use super::ContextManager;

    #[derive(Debug)]
    struct Player {
        name: String,
        pv: i64,
        xp: i64,
        non_relevant_stuff: usize,
    }

    impl Player {
        fn new<T: ToString>(name: T, pv: i64, xp: i64) -> Player {
            Player {
                name: name.to_string(),
                pv: pv,
                xp: xp,
                non_relevant_stuff: 0,
            }
        }

        fn new_rc<T: ToString>(name: T, pv: i64, xp: i64) -> Rc<RefCell<Player>> {
            Rc::new(RefCell::new(Player::new(name, pv, xp)))
        }
    }

    declare_data_binding! {
        Player {
            name,
            pv,
            xp
        }
    }

    #[test]
    fn register_view_player() {
        let mut context = ContextManager::default();
        context.register_view("foo".to_string());
        let player = Player::new_rc("Grub", 42, 100);
        context.register_store("foo", "player".to_string(), &player).unwrap();

        // Not in the correct view
        assert!(context.get_attribute("player.pv").is_none());
        context.switch_to_view("foo".to_string());
        assert_eq!(context.get_attribute("player.pv").unwrap(), StoreValue::Integer(42));
        assert_eq!(context.get_attribute("player.xp").unwrap(), StoreValue::Integer(100));
    }

    #[test]
    fn register_view_value() {
        let mut context = ContextManager::default();
        context.register_view("foo".to_string());
        context.register_value("foo", "option.width".to_string(),
            StoreValue::Integer(42)).unwrap();
        assert!(context.get_attribute("option.width").is_none());
        context.switch_to_view("foo".to_string());
        assert_eq!(context.get_attribute("option.width").unwrap(), StoreValue::Integer(42));
    }

    #[test]
    fn masking_global_value_by_view() {
        let foo = "foo".to_string();
        let bar = "bar".to_string();
        let mut context = ContextManager::default();
        context.register_view(foo.clone());
        context.register_view(bar.clone());
        context.register_view("foobar".to_string());
        context.register_value("foo", "option.width".to_string(),
            StoreValue::String("foo_value".to_string())).unwrap();
        context.register_value("bar", "option.width".to_string(),
            StoreValue::String("bar_value".to_string())).unwrap();
        context.register_global_value("option.width".to_string(),
            StoreValue::String("global_value".to_string()));

        // In view "foobar" -> get global value
        context.switch_to_view("foobar".to_string());
        assert_eq!(context.get_attribute("option.width").unwrap(), StoreValue::String("global_value".to_string()));
        // In view "foo" -> get foo specific value
        context.switch_to_view("foo".to_string());
        assert_eq!(context.get_attribute("option.width").unwrap(), StoreValue::String("foo_value".to_string()));
        // In view "bar" -> get bar specific value
        context.switch_to_view("bar".to_string());
        assert_eq!(context.get_attribute("option.width").unwrap(), StoreValue::String("bar_value".to_string()));
    }

    #[test]
    fn masking_iterator() {
        let mut context = ContextManager::default();
        context.register_view("foo".to_string());
        let global_players = Rc::new(RefCell::new(vec![Player::new("Grub", 1, 11), Player::new("Gnom", 2, 22)]));
        context.register_global_iterator("game.friends".to_string(), &global_players);
        let foo_players = Rc::new(RefCell::new(vec![Player::new("Turtle", 3, 33)]));
        context.register_iterator("foo", "game.friends".to_string(), &foo_players).unwrap();
        let mut global_vec = Vec::new();
        assert!(context.compare_and_update("game.friends", "pv", &mut global_vec).unwrap());
        assert_eq!(global_vec, [StoreValue::Integer(1), StoreValue::Integer(2)]);
        context.switch_to_view("foo".to_string());
        let mut foo_vec = Vec::new();
        assert!(context.compare_and_update("game.friends", "pv", &mut foo_vec).unwrap());
        assert_eq!(foo_vec, [StoreValue::Integer(3)]);
    }
}
