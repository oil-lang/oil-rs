
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

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

    pub fn update(&mut self, vp: Viewport) {
        for v in self.stack.iter_mut() {
            v.borrow_mut().update(vp);
        }
    }

    pub fn add_view(&mut self, name: String, view: View) {
        let rcv = Rc::new(RefCell::new(view));
        // TODO: clean-up name
        if name == "main" {
            self.stack.push(rcv.clone())
        }
        self.views.insert(name, rcv);
    }

    pub fn render_views<C>(&self, ctx: &mut C)
        where C: RenderBackbend
    {

        for v in &self.stack {
            v.borrow().render(ctx);
        }
    }
}
