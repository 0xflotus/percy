pub use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
pub use std::rc::Rc;
use wasm_bindgen::prelude::Closure;
use webapis::*;

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

impl VirtualNode {
    /// Create a new virtual node with a given tag.
    ///
    /// These get patched into the DOM using `document.createElement`
    ///
    /// ```
    /// let div = VirtualNode::tag("div");
    /// ```
    pub fn new(tag: &str) -> VirtualNode {
        let props = HashMap::new();
        let events = Events(HashMap::new());
        VirtualNode {
            tag: tag.to_string(),
            props,
            events,
            children: vec![],
            parent: None,
            text: None,
        }
    }

    /// Create a text node.
    ///
    /// These get patched into the DOM using `document.createTextNode`
    ///
    /// ```
    /// let div = VirtualNode::text("div");
    /// ```
    pub fn text(text: &str) -> VirtualNode {
        VirtualNode {
            tag: "".to_string(),
            props: HashMap::new(),
            events: Events(HashMap::new()),
            children: vec![],
            parent: None,
            text: Some(text.to_string()),
        }
    }
}

impl VirtualNode {
    /// Build a DOM element by recursively creating DOM nodes for this element and it's
    /// children, it's children's children, etc.
    pub fn create_element(&mut self) -> Element {
        let elem = document.create_element(&self.tag);

        self.props.iter().for_each(|(name, value)| {
            elem.set_attribute(name, value);
        });

        self.events.0.iter_mut().for_each(|(onevent, callback)| {
            // onclick -> click
            let event = &onevent[2..];

            let callback = callback.take().unwrap();
            elem.add_event_listener(event, &callback);
            callback.forget();
        });

        self.children.iter_mut().for_each(|child| {
            let mut child = child.borrow_mut();

            if child.text.is_some() {
                elem.append_text_child(document.create_text_node(&child.text.as_ref().unwrap()));
            }

            if child.text.is_none() {
                elem.append_child(child.create_element());
            }
        });

        elem
    }
}

// Used by our html! macro to turn "Strings of text" into virtual nodes.
impl<'a> From<&'a str> for VirtualNode {
    fn from(text: &'a str) -> Self {
        VirtualNode::text(text)
    }
}
impl From<String> for VirtualNode {
    fn from(text: String) -> Self {
        VirtualNode::text(&text)
    }
}
impl<'a> From<&'a String> for VirtualNode {
    fn from(text: &'a String) -> Self {
        VirtualNode::text(text)
    }
}

impl fmt::Debug for VirtualNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "VirtualNode | tag: {}, props: {:#?}, text: {:#?}, children: {:#?} |",
            self.tag, self.props, self.text, self.children
        )
    }
}

impl fmt::Display for VirtualNode {
    // Turn a VirtualNode and all of it's children (recursively) into an HTML string
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.text.is_some() {
            write!(f, "{}", self.text.as_ref().unwrap())
        } else {
            write!(f, "<{}", self.tag).unwrap();

            for (prop, value) in self.props.iter() {
                write!(f, r#" {}="{}""#, prop, value)?;
            }

            write!(f, ">");

            for child in self.children.iter() {
                write!(f, "{}", child.borrow().to_string())?;
            }
            write!(f, "</{}>", self.tag)
        }
    }
}

/// We need a custom implementation of fmt::Debug since Fn() doesn't
/// implement debug.
pub struct Events(pub HashMap<String, Option<Closure<Fn() -> ()>>>);

impl PartialEq for Events {
    // TODO: What should happen here..? And why?
    fn eq(&self, _rhs: &Self) -> bool {
        true
    }
}

impl fmt::Debug for Events {
    // Print out all of the event names for this VirtualNode
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let events: String = self.0.keys().map(|key| format!("{} ", key)).collect();
        write!(f, "{}", events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_string() {
        let node = html! {
        <div id="some-id", !onclick=|| {},>
            <span>
                { "Hello world" }
            </span>
        </div>
        };
        let expected = r#"<div id="some-id"><span>Hello world</span></div>"#;

        assert_eq!(node.to_string(), expected);
    }
}
