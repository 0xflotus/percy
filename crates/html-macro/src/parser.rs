use crate::Attr;
use crate::Tag;
use proc_macro2::Literal;
use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use std::collections::HashMap;
use syn::export::Span;
use syn::group::Group;
use syn::parse::{Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::{braced, parse_macro_input, Block, Expr, Ident, Token};

/// Iterate over Tag's that we've parsed and build a tree of VirtualNode's
pub struct HtmlParser {
    /// As we parse our macro tokens we'll generate new tokens to return back into the compiler
    /// when we're done.
    tokens: Vec<proc_macro2::TokenStream>,
    /// Everytime we encounter a new node we'll use the current_idx to name it.
    /// Then we'll increment the current_idx by one.
    /// This gives every node that we encounter a unique name that we can use to find
    /// it later when we want to push child nodes into parent nodes
    current_idx: usize,
    /// The order that we encountered nodes while parsing.
    node_order: Vec<usize>,
    /// Each time we encounter a new node that could possible be a parent node
    /// we push it's node index onto the stack.
    ///
    /// Text nodes cannot be parent nodes.
    parent_stack: Vec<usize>,
    /// Key -> index of the parent node within the HTML tree
    /// Value -> vector of child node indices
    parent_to_children: HashMap<usize, Vec<usize>>,
}

/// TODO: I've hit a good stopping point... but we can clean these methods up / split them up
/// a bit...
impl HtmlParser {
    pub fn new() -> HtmlParser {
        let mut parent_to_children: HashMap<usize, Vec<usize>> = HashMap::new();
        parent_to_children.insert(0, vec![]);

        HtmlParser {
            tokens: vec![],
            current_idx: 0,
            node_order: vec![],
            parent_stack: vec![],
            parent_to_children,
        }
    }

    pub fn push_tag(&mut self, tag: Tag) {
        let mut idx = &mut self.current_idx;
        let mut parent_stack = &mut self.parent_stack;
        let mut node_order = &mut self.node_order;
        let mut parent_to_children = &mut self.parent_to_children;
        let mut tokens = &mut self.tokens;

        // TODO: Split each of these into functions and make this DRY. Can barely see what's
        // going on.
        match tag {
            Tag::Open { name, attrs } => {
                if *idx == 0 {
                    node_order.push(0);
                    parent_stack.push(0);
                }

                // The root node is named `node_0`. All of it's descendants are node_1.. node_2.. etc.
                // This just comes from the `idx` variable
                // TODO: Not sure what the span is supposed to be so I just picked something..
                let var_name = Ident::new(format!("node_{}", idx).as_str(), name.span());

                let name = format!("{}", name);
                let node = quote! {
                    let mut #var_name = VirtualNode::new(#name);
                };
                tokens.push(node);

                for attr in attrs.iter() {
                    let key = format!("{}", attr.key);
                    let value = &attr.value;
                    match value {
                        Expr::Closure(closure) => {
                            // TODO: Use this to decide Box<FnMut(_, _, _, ...)
                            // After we merge the DomUpdater
                            let arg_count = closure.inputs.len();

                            let add_closure = quote! {
                                #[cfg(target_arch = "wasm32")]
                                {
                                  let closure = wasm_bindgen::prelude::Closure::wrap(
                                      Box::new(#value) as Box<FnMut(_)>
                                  );
                                  let closure = Box::new(closure);
                                  #var_name.events.0.insert(#key.to_string(), closure);
                                }
                            };

                            tokens.push(add_closure);
                        }
                        _ => {
                            let insert_attribute = quote! {
                                #var_name.props.insert(#key.to_string(), #value.to_string());
                            };
                            tokens.push(insert_attribute);
                        }
                    };
                }

                // The first open tag that we see is our root node so we won't worry about
                // giving it a parent
                if *idx == 0 {
                    *idx += 1;
                    return;
                }

                let parent_idx = parent_stack[parent_stack.len() - 1];

                parent_stack.push(*idx);
                node_order.push(*idx);

                parent_to_children
                    .get_mut(&parent_idx)
                    .expect("Parent of this node")
                    .push(*idx);

                parent_to_children.insert(*idx, vec![]);

                *idx += 1;
            }
            Tag::Close { name } => {
                parent_stack.pop();
            }
            Tag::Text { text } => {
                if *idx == 0 {
                    node_order.push(0);
                    parent_stack.push(0);
                }

                // TODO: Figure out how to use spans
                let var_name = Ident::new(format!("node_{}", idx).as_str(), Span::call_site());

                let text_node = quote! {
                    let mut #var_name = VirtualNode::text(#text);
                };

                tokens.push(text_node);

                if *idx == 0 {
                    *idx += 1;
                    return;
                }

                let parent_idx = parent_stack[parent_stack.len() - 1];

                node_order.push(*idx);

                parent_to_children
                    .get_mut(&parent_idx)
                    .expect("Parent of this text node")
                    .push(*idx);

                *idx += 1;
            }
            Tag::Braced { block } => block.stmts.iter().for_each(|stmt| {
                if *idx == 0 {
                    // Here we handle a block being the root node of an `html!` call
                    //
                    // html { { some_node }  }
                    let node = quote! {
                        let node_0 = #stmt;
                    };
                    tokens.push(node);
                } else {
                    // Here we handle a block being a descendant within some html! call
                    //
                    // html { <div> { some_node } </div> }

                    let node_name = format!("node_{}", idx);
                    let node_name = Ident::new(node_name.as_str(), stmt.span());

                    let nodes = quote! {
                        let #node_name = #stmt;
                    };
                    tokens.push(nodes);

                    let parent_idx = parent_stack[parent_stack.len() - 1];

                    parent_to_children
                        .get_mut(&parent_idx)
                        .expect("Parent of this text node")
                        .push(*idx);
                    node_order.push(*idx);

                    *idx += 1;
                }
            }),
        };
    }

    ///  1. Pop a node off the stack
    ///  2. Look up all of it's children in parent_to_children
    ///  3. Append the children to this node
    ///  4. Move on to the next node (as in, go back to step 1)
    pub fn finish(&mut self) -> proc_macro2::TokenStream {
        let mut idx = &mut self.current_idx;
        let mut parent_stack = &mut self.parent_stack;
        let mut node_order = &mut self.node_order;
        let mut parent_to_children = &mut self.parent_to_children;
        let mut tokens = &mut self.tokens;

        if node_order.len() > 1 {
            for _ in 0..(node_order.len()) {
                let parent_idx = node_order.pop().unwrap();

                // TODO: Figure out how to really use spans
                let parent_name =
                    Ident::new(format!("node_{}", parent_idx).as_str(), Span::call_site());

                let parent_to_children_indices = match parent_to_children.get(&parent_idx) {
                    Some(children) => children,
                    None => continue,
                };

                if parent_to_children_indices.len() > 0 {
                    let create_children_vec = quote! {
                        #parent_name.children = Some(vec![]);
                    };

                    tokens.push(create_children_vec);

                    for child_idx in parent_to_children_indices.iter() {
                        let children =
                            Ident::new(format!("node_{}", child_idx).as_str(), Span::call_site());

                        // TODO: Multiple .as_mut().unwrap() of children. Let's just do this once.
                        let push_children = quote! {
                            for child in #children.into_iter() {
                                #parent_name.children.as_mut().unwrap().push(child);
                            }
                        };
                        tokens.push(push_children);
                    }
                }
            }
        }

        // Create a virtual node tree
        quote! {
            {
                #(#tokens)*
                // Root node is always named node_0
                node_0
            }
        }
    }
}
