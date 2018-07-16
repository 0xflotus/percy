#[macro_use]
extern crate virtual_dom_rs;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use virtual_dom_rs::virtual_node::VirtualNode;

mod state;
pub use state::*;

pub use virtual_dom_rs::webapis::*;

pub struct App {
    pub state: Rc<RefCell<State>>,
    previous_vdom: Option<VirtualNode>
}

impl App {
    pub fn new() -> App {
        App {
            state: Rc::new(RefCell::new(State::new())),
            previous_vdom: None
        }
    }

    // TODO: Just use `new(config: AppConfig)` and pass in state json Option
    pub fn from_state_json(json: &str) -> App {
        App {
            state: Rc::new(RefCell::new(State::from_json(json))),
            previous_vdom: None
        }
    }
}

impl App {
    pub fn render(&self) -> VirtualNode {
        let state = Rc::clone(&self.state);
        let click_count = self.state.borrow().click_count();
        let click_count = click_count.to_string();

        let click_count = html! { <strong style="font-size: 30px",>{ click_count }</strong> };

        html! {
        <div>
          <span> { "The button has been clicked: " click_count  " times!"} </span>
          <button !onclick=move|| { state.borrow_mut().msg(Msg::Click) },>{ "Click me!" }</button>
        </div>
        }
    }

    pub fn update_dom (&mut self, root_node: &Element) {
        let mut new_vdom = self.render();

        if let Some(ref previous_vdom) = self.previous_vdom {
            let patches = virtual_dom_rs::diff(&previous_vdom, &mut new_vdom);
            virtual_dom_rs::patch(&root_node, &patches);
        }

        self.previous_vdom = Some(new_vdom);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn click_msg() {
        let mut app = App::new();

        assert_eq!(app.state.borrow().click_count(), 0);
        app.state.borrow_mut().msg(Msg::Click);
        assert_eq!(app.state.borrow().click_count(), 1);
    }

    #[test]
    fn serialize_deserialize() {}
}
