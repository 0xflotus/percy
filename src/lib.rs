//!

use std::collections::HashMap;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Default))]
pub struct VirtualNode {
    tag: String,
    props: HashMap<String, String>,
    events: HashMap<String, fn() -> ()>,
    children: Vec<VirtualNode>,
    /// Some(String) if this is a [text node](https://developer.mozilla.org/en-US/docs/Web/API/Text).
    /// When patching these into a real DOM these use `document.createTextNode(text)`
    text: Option<String>,
}

impl VirtualNode {
    fn new (tag: &str) -> VirtualNode {
        let props = HashMap::new();
        let events = HashMap::new();
        VirtualNode {
            tag: tag.to_string(),
            props,
            events,
            children: vec![],
            text: None
        }
    }
}

pub fn createElement(node: &VirtualNode) {
    // document.createElement(node.type)
}

struct ParsedNodeTracker<'a> {
    current_node: Option<VirtualNode>,
    parent_node: Option<&'a VirtualNode>,
}

// TODO: Move to html_macro.rs along w/ tests
#[macro_export]
macro_rules! html {
    ($($remaining_html:tt)*) => {{
        let mut pnt = ParsedNodeTracker {
            current_node: None,
            parent_node: None
        };

        recurse_html! { pnt $($remaining_html)* };

        pnt.current_node.unwrap()
    }};
}

#[macro_export]
macro_rules! recurse_html {
    // The beginning of an element without any attributrs.
    // For <div></div> this is
    // <div>
    ($pnt:ident < $start_tag:ident > $($remaining_html:tt)*) => {
        let current_node = VirtualNode::new(stringify!($start_tag));
        $pnt.current_node = Some(current_node);

        recurse_html! { $pnt $($remaining_html)* }
    };

    // The beginning of an element.
    // For <div id="10",> this is
    // <div
    ($pnt:ident < $start_tag:ident $($remaining_html:tt)*) => {
        let current_node = VirtualNode::new(stringify!($start_tag));
        $pnt.current_node = Some(current_node);

        recurse_html! { $pnt $($remaining_html)* }
    };

    // The end of an opening tag.
    // For <div id="10",> this is:
    //  >
    ($pnt:ident > $($remaining_html:tt)*) => {
        println!("opening tag");

        recurse_html! { $pnt $($remaining_html)* }
    };

    // A property
    // For <div id="10",> this is:
    // id = "10",
    ($pnt:ident $prop_name:tt = $prop_value:expr, $($remaining_html:tt)*) => {
        $pnt.current_node.as_mut().unwrap().props.insert(
            stringify!($prop_name).to_string(),
            $prop_value.to_string()
        );

        recurse_html! { $pnt $($remaining_html)* }
    };

    // An event
    // for <div $onclick=|| { do.something(); },></div> ths is:
    //   $onclick=|| { do.something() }
    ($pnt:ident ! $event_name:tt = $callback:expr, $($remaining_html:tt)*) => {
        $pnt.current_node.as_mut().unwrap().events.insert(
            stringify!($event_name).to_string(),
            $callback
        );

        recurse_html! { $pnt $($remaining_html)* }
    };


    // A closing tag for some associated opening tag name
    // For <div id="10",></div> this is:
    // </div>
    ($pnt:ident < / $end_tag:ident > $($remaining_html:tt)*) => {
        recurse_html! { $pnt $($remaining_html)* }
    };

    // No more HTML remaining. We're done!
    ($pnt:ident) => {
    };

    // TODO: README explains that props must end with commas
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_div() {
        let node = html!{
        <div></div>
        };

        let expected_node = VirtualNode {
            tag: "div".to_string(),
            ..VirtualNode::default()
        };

        assert_eq!(node, expected_node);
    }

    #[test]
    fn one_prop() {
        let node = html!{
        <div id="hello-world",></div>
        };

        let mut props = HashMap::new();
        props.insert("id".to_string(), "hello-world".to_string());
        let expected_node = VirtualNode {
            tag: "div".to_string(),
            props,
            ..VirtualNode::default()
        };

        assert_eq!(node, expected_node);
    }

    #[test]
    fn event() {
        let mut closure_ran = false;

        let node = html!{
        <div !onclick=|| {closure_ran = true},></div>
        };

        node.events.get("!onclick").unwrap()();

        assert!(closure_ran);
    }
}
