use crate::Patch;
use crate::VirtualNode;
use std::cmp::min;
use std::collections::HashMap;
use std::mem;

/// Given two VirtualNode's generate Patch's that would turn the old virtual node's
/// real DOM node equivalent into the new VirtualNode's real DOM node equivalent.
pub fn diff<'a>(old: &'a VirtualNode, new: &'a VirtualNode) -> Vec<Patch<'a>> {
    diff_recursive(&old, &new, &mut 0)
}

fn diff_recursive<'a, 'b>(
    old: &'a VirtualNode,
    new: &'a VirtualNode,
    cur_node_idx: &'b mut usize,
) -> Vec<Patch<'a>> {
    let mut patches = vec![];
    let mut replace = false;

    // Different enum variants, replace!
    if mem::discriminant(old) != mem::discriminant(new) {
        replace = true;
    }

    // Different element tags, replace!
    if let (VirtualNode::Element(old_element), VirtualNode::Element(new_element)) = (old, new) {
        if old_element.tag != new_element.tag {
            replace = true;
        }
    }

    // Handle replacing of a node
    if replace {
        patches.push(Patch::Replace(*cur_node_idx, &new));
        if let VirtualNode::Element(old_element_node) = old {
            for child in old_element_node.children.iter() {
                increment_node_idx_for_children(child, cur_node_idx);
            }
        }
        return patches;
    }

    // The following comparison can only contain identical variants, other
    // cases have already been handled above by comparing variant
    // discriminants.
    match (old, new) {
        // We're comparing two text nodes
        (VirtualNode::Text(old_text), VirtualNode::Text(new_text)) => {
            if old_text != new_text {
                patches.push(Patch::ChangeText(*cur_node_idx, &new_text));
            }
        }

        // We're comparing two element nodes
        (VirtualNode::Element(old_element), VirtualNode::Element(new_element)) => {
            let mut add_attributes: HashMap<&str, &str> = HashMap::new();
            let mut remove_attributes: Vec<&str> = vec![];

            // TODO: -> split out into func
            for (new_prop_name, new_prop_val) in new_element.props.iter() {
                match old_element.props.get(new_prop_name) {
                    Some(ref old_prop_val) => {
                        if old_prop_val != &new_prop_val {
                            add_attributes.insert(new_prop_name, new_prop_val);
                        }
                    }
                    None => {
                        add_attributes.insert(new_prop_name, new_prop_val);
                    }
                };
            }

            // TODO: -> split out into func
            for (old_prop_name, old_prop_val) in old_element.props.iter() {
                if add_attributes.get(&old_prop_name[..]).is_some() {
                    continue;
                };

                match new_element.props.get(old_prop_name) {
                    Some(ref new_prop_val) => {
                        if new_prop_val != &old_prop_val {
                            remove_attributes.push(old_prop_name);
                        }
                    }
                    None => {
                        remove_attributes.push(old_prop_name);
                    }
                };
            }

            if add_attributes.len() > 0 {
                patches.push(Patch::AddAttributes(*cur_node_idx, add_attributes));
            }
            if remove_attributes.len() > 0 {
                patches.push(Patch::RemoveAttributes(*cur_node_idx, remove_attributes));
            }

            let old_child_count = old_element.children.len();
            let new_child_count = new_element.children.len();

            if new_child_count > old_child_count {
                let append_patch: Vec<&'a VirtualNode> = new_element.children[old_child_count..].iter().collect();
                patches.push(Patch::AppendChildren(*cur_node_idx, append_patch))
            }

            if new_child_count < old_child_count {
                patches.push(Patch::TruncateChildren(*cur_node_idx, new_child_count))
            }

            let min_count = min(old_child_count, new_child_count);
            for index in 0..min_count {
                *cur_node_idx = *cur_node_idx + 1;
                let old_child = &old_element.children[index];
                let new_child = &new_element.children[index];
                patches.append(&mut diff_recursive(&old_child, &new_child, cur_node_idx))
            }
            if new_child_count < old_child_count {
                for child in old_element.children[min_count..].iter() {
                    increment_node_idx_for_children(child, cur_node_idx);
                }
            }
        }
        (VirtualNode::Text(_), VirtualNode::Element(_)) |
        (VirtualNode::Element(_), VirtualNode::Text(_)) => {
            unreachable!("Unequal variant discriminants should already have been handled");
        }
    };

    //    new_root.create_element()
    patches
}

fn increment_node_idx_for_children<'a, 'b>(old: &'a VirtualNode, cur_node_idx: &'b mut usize) {
    *cur_node_idx += 1;
    if let VirtualNode::Element(element_node) = old {
        for child in element_node.children.iter() {
            increment_node_idx_for_children(&child, cur_node_idx);
        }
    }
}

#[cfg(test)]
mod diff_test_case;
#[cfg(test)]
use self::diff_test_case::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{html, VirtualNode, VText};
    use std::collections::HashMap;

    #[test]
    fn replace_node() {
        DiffTestCase {
            old: html! { <div> </div> },
            new: html! { <span> </span> },
            expected: vec![Patch::Replace(0, &html! { <span></span> })],
            description: "Replace the root if the tag changed",
        }
        .test();
        DiffTestCase {
            old: html! { <div> <b></b> </div> },
            new: html! { <div> <strong></strong> </div> },
            expected: vec![Patch::Replace(1, &html! { <strong></strong> })],
            description: "Replace a child node",
        }
        .test();
        DiffTestCase {
            old: html! { <div> <b>1</b> <b></b> </div> },
            new: html! { <div> <i>1</i> <i></i> </div>},
            expected: vec![
                Patch::Replace(1, &html! { <i>1</i> }),
                Patch::Replace(3, &html! { <i></i> }),
            ], //required to check correct index
            description: "Replace node with a child",
        }
        .test();
    }

    #[test]
    fn add_children() {
        DiffTestCase {
            old: html! { <div> <b></b> </div> },
            new: html! { <div> <b></b> <new></new> </div> },
            expected: vec![Patch::AppendChildren(0, vec![&html! { <new></new> }])],
            description: "Added a new node to the root node",
        }
        .test();
    }

    #[test]
    fn remove_nodes() {
        DiffTestCase {
            old: html! { <div> <b></b> <span></span> </div> },
            new: html! { <div> </div> },
            expected: vec![Patch::TruncateChildren(0, 0)],
            description: "Remove all child nodes at and after child sibling index 1",
        }
        .test();
        DiffTestCase {
            old: html! {
            <div>
             <span>
               <b></b>
               // This `i` tag will get removed
               <i></i>
             </span>
             // This `strong` tag will get removed
             <strong></strong>
            </div> },
            new: html! {
            <div>
             <span>
              <b></b>
             </span>
            </div> },
            expected: vec![Patch::TruncateChildren(0, 1), Patch::TruncateChildren(1, 1)],
            description: "Remove a child and a grandchild node",
        }
        .test();
        DiffTestCase {
            old: html! { <div> <b> <i></i> <i></i> </b> <b></b> </div> },
            new: html! { <div> <b> <i></i> </b> <i></i> </div>},
            expected: vec![
                Patch::TruncateChildren(1, 1),
                Patch::Replace(4, &html! { <i></i> }),
            ], //required to check correct index
            description: "Removing child and change next node after parent",
        }
        .test();
    }

    #[test]
    fn add_attributes() {
        let mut attributes = HashMap::new();
        attributes.insert("id", "hello");

        DiffTestCase {
            old: html! { <div> </div> },
            new: html! { <div id="hello"> </div> },
            expected: vec![Patch::AddAttributes(0, attributes.clone())],
            description: "Add attributes",
        }
        .test();

        DiffTestCase {
            old: html! { <div id="foobar"> </div> },
            new: html! { <div id="hello"> </div> },
            expected: vec![Patch::AddAttributes(0, attributes)],
            description: "Change attribute",
        }
        .test();
    }

    #[test]
    fn remove_attributes() {
        DiffTestCase {
            old: html! { <div id="hey-there"></div> },
            new: html! { <div> </div> },
            expected: vec![Patch::RemoveAttributes(0, vec!["id"])],
            description: "Add attributes",
        }
        .test();
    }

    #[test]
    fn change_attribute() {
        let mut attributes = HashMap::new();
        attributes.insert("id", "changed");

        DiffTestCase {
            old: html! { <div id="hey-there"></div> },
            new: html! { <div id="changed"> </div> },
            expected: vec![Patch::AddAttributes(0, attributes)],
            description: "Add attributes",
        }
        .test();
    }

    #[test]
    fn replace_text_node() {
        DiffTestCase {
            old: html! { Old },
            new: html! { New },
            expected: vec![Patch::ChangeText(0, &VText::new("New"))],
            description: "Replace text node",
        }
        .test();
    }

    //    // TODO: Key support
    //    #[test]
    //    fn reorder_chldren() {
    //        let mut attributes = HashMap::new();
    //        attributes.insert("class", "foo");
    //
    //        let old_children = vec![
    //            // old node 0
    //            html! { <div key="hello", id="same-id", style="",></div> },
    //            // removed
    //            html! { <div key="gets-removed",> { "This node gets removed"} </div>},
    //            // old node 2
    //            html! { <div key="world", class="changed-class",></div>},
    //            // removed
    //            html! { <div key="this-got-removed",> { "This node gets removed"} </div>},
    //        ];
    //
    //        let new_children = vec![
    //            html! { <div key="world", class="foo",></div> },
    //            html! { <div key="new",> </div>},
    //            html! { <div key="hello", id="same-id",></div>},
    //        ];
    //
    //        test(DiffTestCase {
    //            old: html! { <div> { old_children } </div> },
    //            new: html! { <div> { new_children } </div> },
    //            expected: vec![
    //                // TODO: Come up with the patch structure for keyed nodes..
    //                // keying should only work if all children have keys..
    //            ],
    //            description: "Add attributes",
    //        })
    //    }

}
