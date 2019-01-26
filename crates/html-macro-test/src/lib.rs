#![feature(proc_macro_hygiene)]

use html_macro::h;
use std::collections::HashMap;
use virtual_node::VirtualNode;

struct HtmlMacroTest {
    desc: &'static str,
    generated: VirtualNode,
    expected: VirtualNode,
}

impl HtmlMacroTest {
    fn test(self) {
        assert_eq!(self.generated, self.expected, "{}", self.desc);

        for (index, child) in self.expected.children.as_ref().unwrap().iter().enumerate() {
            assert_eq!(
                child,
                &self.generated.children.as_ref().unwrap()[index],
                "{}",
                self.desc
            );
        }
    }
}

#[test]
fn empty_div() {
    HtmlMacroTest {
        generated: html! { <div></div> },
        expected: VirtualNode::new("div"),
        desc: "Empty div",
    }
    .test();
}

#[test]
fn one_prop() {
    let mut props = HashMap::new();
    props.insert("id".to_string(), "hello-world".to_string());
    let mut expected = VirtualNode::new("div");
    expected.props = props;

    HtmlMacroTest {
        generated: html! { <div id="hello-world"></div> },
        expected,
        desc: "One property",
    }
    .test();
}

#[test]
fn event() {
    HtmlMacroTest {
        generated: html! {
            <div onclick=||{}></div>
        },
        expected: html! {<div></div>},
        desc: "Events are ignored in non wasm-32 targets",
    }
    .test();
}

#[test]
fn child_node() {
    let mut expected = VirtualNode::new("div");
    expected.children = Some(vec![VirtualNode::new("span")]);

    HtmlMacroTest {
        generated: html! { <div><span></span></div> },
        expected,
        desc: "Child node",
    }
    .test();
}

#[test]
fn sibling_child_nodes() {
    let mut expected = VirtualNode::new("div");
    expected.children = Some(vec![VirtualNode::new("span"), VirtualNode::new("b")]);

    HtmlMacroTest {
        generated: html! { <div><span></span><b></b></div> },
        expected,
        desc: "Sibling child nodes",
    }
    .test();
}

#[test]
fn three_nodes_deep() {
    let mut child = VirtualNode::new("span");
    child.children = Some(vec![VirtualNode::new("b")]);

    let mut expected = VirtualNode::new("div");
    expected.children = Some(vec![child]);

    HtmlMacroTest {
        generated: html! { <div><span><b></b></span></div> },
        expected,
        desc: "Nested 3 nodes deep",
    }
    .test()
}

#[test]
fn sibling_text_nodes() {
    let mut expected = VirtualNode::new("div");
    expected.children = Some(vec![
        VirtualNode::text("This is a text node")
    ]);

    HtmlMacroTest {
        generated: html! { <div>This is a text node</div> },
        expected,
        desc: "Nested text node",
    }
        .test();
}


    #[test]
    fn nested_macro() {
        let child_2 = html! { <b></b> };

        let mut expected = VirtualNode::new("div");
        expected.children = Some(vec![VirtualNode::new("span"), VirtualNode::new("b")]);

        HtmlMacroTest {
            generated: html! {
              <div>
                { html! { <span></span> } }
                { child_2 }
              </div>
            },
            expected,
            desc: "Nested macros",
        }.test();
    }

    #[test]
    fn vec_of_nodes() {
        let children = vec![html! { <div> </div>}, html! { <strong> </strong>}];

        let mut expected = VirtualNode::new("div");
        expected.children = Some(vec![VirtualNode::new("div"), VirtualNode::new("strong")]);

        HtmlMacroTest {
            generated: html! { <div> { children } </div> },
            expected,
            desc: "Vec of nodes",
        }.test();
    }


    #[test]
    fn text_root_node() {
        HtmlMacroTest {
            generated: html! { some text },
            expected: VirtualNode::text("some text"),
            desc: "Text as root node",
        }.test()
    }

// TODO: Test for self closing tags such as <b />
