//! Kept in its own file to more easily import into the book

use console_error_panic_hook;

use virtual_dom_rs::prelude::*;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A test case that both diffing and patching are working in a real browser
pub struct DiffPatchTest<'a> {
    pub desc: &'static str,
    pub old: VirtualNode,
    pub new: VirtualNode,
    pub override_expected: Option<&'a str>,
}

impl<'a> DiffPatchTest<'a> {
    pub fn test(&mut self) {
        console_error_panic_hook::set_once();

        let document = web_sys::window().unwrap().document().unwrap();

        // If we haven't set an id for our element we hash the description of the test and set
        // that as the ID.
        // We need an ID in order to find the element within the DOM, otherwise we couldn't run
        // our assertions.
        if self.old.props.get("id").is_none() {
            let mut hashed_desc = DefaultHasher::new();

            self.desc.hash(&mut hashed_desc);

            self.old
                .props
                .insert("id".to_string(), hashed_desc.finish().to_string());
        }

        // Add our old node into the DOM
        let root_node = self.old.create_element().element;
        document.body().unwrap().append_child(&root_node).unwrap();

        let elem_id = self.old.props.get("id").unwrap().clone();

        // This is our root node that we're about to patch.
        // It isn't actually patched yet.. but by the time we use this it will be.
        let patched_element = document.get_element_by_id(&elem_id).unwrap();

        let patches = virtual_dom_rs::diff(&self.old, &self.new);

        virtual_dom_rs::patch(root_node, &patches);

        let expected_new_root_node = self.new.to_string();
        let mut expected_new_root_node = expected_new_root_node.as_str();

        if let Some(ref expected) = self.override_expected {
            expected_new_root_node = expected;
        }

        web_sys::console::log_1(&format!("NEW NODE {:#?}", patched_element.outer_html()).into());
        web_sys::console::log_1(&format!("Outter HTML {:#?}", expected_new_root_node).into());

        assert_eq!(
            &patched_element.outer_html(),
            &expected_new_root_node,
            "{}",
            self.desc
        );
    }
}
