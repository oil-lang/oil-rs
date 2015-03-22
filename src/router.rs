
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use markup::MAIN_VIEW_NAME;
use markup::Library;
use style::Stylesheet;
use RenderBackbend;
use View;
use Viewport;

pub struct Router {
    stack: Vec<Rc<RefCell<View>>>,
    views: HashMap<String, Rc<RefCell<View>>>,
}

impl Router {

    pub fn new() -> Router {
        Router {
            stack: Vec::new(),
            views: HashMap::new(),
        }
    }

    pub fn from_library_and_stylesheet<E>(lib: Library<E>, style: &Stylesheet)
        -> Router
    {
        let mut router = Router::new();
        for (name, view) in lib.views.into_iter() {
            router.add_view(name, View::new(&view, style));
        }
        router
    }

    pub fn update(&mut self, vp: Viewport) {
        for v in self.stack.iter_mut() {
            v.borrow_mut().update(vp);
        }
    }

    pub fn add_view<S : ToString>(&mut self, name: S, view: View) {
        let name_str = name.to_string();
        let rcv = Rc::new(RefCell::new(view));
        if name_str == MAIN_VIEW_NAME {
            self.stack.push(rcv.clone())
        }
        self.views.insert(name_str, rcv);
    }

    pub fn render_views<C>(&self, ctx: &mut C)
        where C: RenderBackbend
    {

        for v in &self.stack {
            v.borrow().render(ctx);
        }
    }
}
