
use std::collections::hash_map::{HashMap,Keys};
use std::rc::Rc;
use std::cell::RefCell;
use glium::Display;

use markup::MAIN_VIEW_NAME;
use markup::Library;
use resource::ResourceManager;
use uil_shared::style::Stylesheet;
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

    pub fn iter_name_views(&self) -> Keys<String,Rc<RefCell<View>>> {
        self.views.keys()
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

    pub fn focus_up(&mut self) {
        if let Some(&mut (_, ref mut view)) = self.stack.last_mut() {
            view.borrow_mut().focus_up();
        }
    }

    pub fn focus_right(&mut self) {
        if let Some(&mut (_, ref mut view)) = self.stack.last_mut() {
            view.borrow_mut().focus_right();
        }
    }

    pub fn focus_left(&mut self) {
        if let Some(&mut (_, ref mut view)) = self.stack.last_mut() {
            view.borrow_mut().focus_left();
        }
    }

    pub fn focus_down(&mut self) {
        if let Some(&mut (_, ref mut view)) = self.stack.last_mut() {
            view.borrow_mut().focus_down();
        }
    }

    pub fn from_library_and_stylesheet<R, E>(
        display: &Display,
        resource_manager:  &R,
        lib: Library<E>,
        style: &Stylesheet)
        -> Router
        where R: ResourceManager
    {
        let mut router = Router::new();
        for (name, view) in lib.views.into_iter() {
            router.add_view(name, View::new(display, resource_manager, &view, style));
        }
        router
    }

    pub fn update(&mut self, display: &Display, vp: Viewport) {
        for &mut (_, ref mut v) in self.stack.iter_mut() {
            v.borrow_mut().update(display, vp);
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

    pub fn render_views<R, C>(
        &self,
        ctx: &C,
        frame: &mut C::Frame,
        resource_manager: &R)
        where C: RenderBackbend,
              R: ResourceManager
    {
        for &(_, ref v) in &self.stack {
            v.borrow().render(ctx, resource_manager, frame);
        }
    }
}
