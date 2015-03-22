
use std::collections::HashMap;
use std::rc::Rc;
use RenderContext;
use View;

pub struct Router {
    stack: Vec<Rc<View>>,
    views: HashMap<String, Rc<View>>,
}

impl Router {

    pub fn new() -> Router {
        Router {
            stack: Vec::new(),
            views: HashMap::new(),
        }
    }

    pub fn add_view(&mut self, name: String, view: View) {
        let rcv = Rc::new(view);
        // TODO: clean-up name
        if name == "main" {
            self.stack.push(rcv.clone())
        }
        self.views.insert(name, rcv);
    }

    pub fn render_views<C>(&self, ctx: &mut C)
        where C: RenderContext
    {

        for v in &self.stack {
            v.render(ctx);
        }
    }
}
