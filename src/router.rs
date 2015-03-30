
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use glium::Display;

use markup::MAIN_VIEW_NAME;
use markup::Library;
use resource::ResourceManager;
use style::Stylesheet;
use RenderBackbend;
use View;
use Viewport;

pub struct Router {
    stack: Vec<(String, Rc<RefCell<View>>)>,
    views: HashMap<String, Rc<RefCell<View>>>,
}

impl Router {

    pub fn new() -> Router {
        Router {
            stack: Vec::new(),
            views: HashMap::new(),
        }
    }

    pub fn goto_view(&mut self, name: String) -> Result<(), &str> {
        match self.views.get(&name) {
            Some(view) => {
                // Look for the view in the stack
                // and pop others views.
                if let Some(pos) = self.stack.iter().rposition(|&(ref n, _)| *n == name) {
                    self.stack.truncate(pos+1);
                } else {
                // If not found then add it to the stack
                    self.stack.push((name, view.clone()));
                }
                Ok(())
            }
            None => Err("View not found")
        }
    }

    pub fn from_library_and_stylesheet<E>(
        display: &Display,
        resource_manager:  &ResourceManager,
        lib: Library<E>,
        style: &Stylesheet)
        -> Router
    {
        let mut router = Router::new();
        for (name, view) in lib.views.into_iter() {
            router.add_view(name, View::new(display, resource_manager, &view, style));
        }
        router
    }

    pub fn update(&mut self, vp: Viewport) {
        for &mut (_, ref mut v) in self.stack.iter_mut() {
            v.borrow_mut().update(vp);
        }
    }

    pub fn add_view<S : ToString>(&mut self, name: S, view: View) {
        let name_str = name.to_string();
        let rcv = Rc::new(RefCell::new(view));
        if name_str == MAIN_VIEW_NAME {
            self.stack.push((name_str.clone(), rcv.clone()));
        }
        self.views.insert(name_str, rcv);
    }

    pub fn render_views<C>(&self, ctx: &mut C, resource_manager: &ResourceManager)
        where C: RenderBackbend
    {
        let mut f = ctx.prepare_frame();
        for &(_, ref v) in &self.stack {
            v.borrow().render(ctx, resource_manager, &mut f);
        }
        ctx.flush_frame(f);
    }
}
