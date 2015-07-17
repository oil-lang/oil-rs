use std::collections::HashMap;

use {
    StoreValue,
    Store,
    PropertyAccessor,
    AttributeSetResult,
    AttributeGetResult
};
use super::Context;

/// The `ContextManager` is templated by a class implementing
/// the `Store` trait. It will then be your only way to store
/// values that can be bind and used in views.
#[derive(Default)]
pub struct ContextManager<G: Default, V> {
    global: G,
    views: HashMap<String, V>,
}

impl<G, V> ContextManager<G, V> 
    where G: Store + Default,
          V: Store
{

    /// Create a new context manager using the specified model.
    ///
    pub fn new(store: G) -> ContextManager<G, V> {
        ContextManager {
            global: store,
            views: HashMap::new()
        }
    }
    
    /// Insert a root `Store` for the given view.
    /// **Note:**
    ///     With the `ContextManager`, all views have a store of the same type.
    ///     But if this is not what you want, you can implement
    ///     the trait `...` TODO(Nemikolh)
    pub fn insert_view_level_store(&mut self, view_name: String, store: V) {
        self.views.insert(view_name, store);
    }
    
    /// Equivalent to the get_attribute of the `Store` trait.
    pub fn get_attribute<'a>(&'a mut self, view: &str, key: &str) -> Option<StoreValue<'a>> {
        if let AttributeGetResult::Found(sv) = self.views.get(view)
            .unwrap()
            .get_attribute(PropertyAccessor::new(key)) {
            Some(sv)
        } else if let AttributeGetResult::Found(sv) = self.global
            .get_attribute(PropertyAccessor::new(key)) {
            Some(sv)
        } else {
            None
        }
    }
    
    /// Equivalent to the `set_attribute` of the `Store` trait.
    /// TODO(Nemikolh): Move that comment for the trait `...`
    /// The view argument can be ignored. It offers the possibility
    /// To have different root stores per view.
    pub fn set_attribute<'a>(&mut self, view: &str, key: &str, value: StoreValue<'a>) {
        if let AttributeSetResult::NoSuchProperty(sv) = self.views.get_mut(view)
            .unwrap()
            .set_attribute(PropertyAccessor::new(key), value) {
            if let AttributeSetResult::NoSuchProperty(_) = self.global
                .set_attribute(PropertyAccessor::new(key), sv) {
                // Insertion failed.
            }
        }
    }
}

/// Implement `Context` equivalent method per view,
/// if the type `V` implement `Context`. 
impl<G, V> ContextManager<G, V>
    where V: Context,
          G: Default 
{
    pub fn register_store_for_view<S: Store>(
        &mut self,
        view_name: String,
        store_name: String,
        store: S) {
        self.views.get_mut(&view_name).unwrap().register_store(store_name, store);
    }
    
    pub fn register_value_for_view<M: Into<StoreValue<'static>>>(
        &mut self,
        view_name: String,
        store_name: String,
        value: M) {
        self.views.get_mut(&view_name).unwrap().register_value(store_name, value);
    }
}

/// Implement `Context` equivalent method for the global data,
/// if the type `G` implement `Context`. 
impl<G, V> ContextManager<G, V>
    where G: Context + Default
{
    pub fn register_global_store<S: Store>(
        &mut self,
        store_name: String,
        store: S)
    {
        self.global.register_store(store_name, store);
    }


    pub fn register_global_value<M: Into<StoreValue<'static>>>(
        &mut self,
        store_name: String,
        value: M)
    {
        self.global.register_value(store_name, value);
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
    use data_bindings::Store;
    use data_bindings::ArrStore;
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
