//!

use std::collections::HashMap;
use std::fmt;
pub use std::cell::RefCell;
pub use std::rc::Rc;
use std::str::FromStr;

#[macro_use]
pub mod html_macro;
pub use html_macro::*;

// TODO: virtual_node.rs module
#[derive(PartialEq)]
pub struct VirtualNode {
    pub tag: String,
    pub props: HashMap<String, String>,
    pub events: Events,
    pub children: Vec<Rc<RefCell<VirtualNode>>>,
    /// We keep track of parents during the `html!` macro in order to be able to crawl
    /// up the tree and assign newly found nodes to the proper parent.
    /// By the time an `html!` macro finishes all nodes will have `parent` None
    pub parent: Option<Rc<RefCell<VirtualNode>>>,
    /// Some(String) if this is a [text node](https://developer.mozilla.org/en-US/docs/Web/API/Text).
    /// When patching these into a real DOM these use `document.createTextNode(text)`
    pub text: Option<String>,
}

impl<'a> From<&'a str> for VirtualNode {
    fn from(text: &'a str) -> Self {
        VirtualNode::text(text)
    }
}

impl fmt::Debug for VirtualNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VirtualNode | tag: {}, props: {:#?}, text: {:#?}, children: {:#?} |", self.tag, self.props, self.text, self.children)
    }
}

/// We need a custom implementation of fmt::Debug since Fn() doesn't
/// implement debug.
pub struct Events(pub HashMap<String, Box<Fn() -> ()>>);

impl PartialEq for Events {
    // TODO: What should happen here..? And why?
    fn eq(&self, rhs: &Self) -> bool {
       true
    }
}

impl fmt::Debug for Events {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let events: String = self.0.keys().map(|key| format!("{} ", key)).collect();
        write!(f, "{}", events)
    }
}

impl VirtualNode {
    pub fn new (tag: &str) -> VirtualNode {
        let props = HashMap::new();
        let events = Events(HashMap::new());
        VirtualNode {
            tag: tag.to_string(),
            props,
            events,
            children: vec![],
            parent: None,
            text: None
        }
    }

    pub fn text (text: &str) -> VirtualNode {
        VirtualNode {
            tag: "".to_string(),
            props: HashMap::new(),
            events: Events(HashMap::new()),
            children: vec![],
            parent: None,
            text: Some(text.to_string())
        }
    }
}
