use virtual_node::VirtualNode;

/// A trait with common functionality for rendering front-end views.
pub trait View {
    /// Render a VirtualNode
    fn render(&self) -> VirtualNode;
}
