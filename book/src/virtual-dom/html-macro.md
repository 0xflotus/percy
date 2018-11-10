# Writing html!

### Text

Text is rendered inside of a block `{}`

```rust
let view = html!{
  <div> {"Text goes here,"} {"or here" " or here!"}</div>
};
```

### Attributes

At this time attributes must end with a `,` due to how our `html!` macro works.

```rust
let view = html!{
  <div id='my-id',></div>
};
```

### Event Handlers

We're currently adding support for all of the standard event handlers. If you run into an error trying to use
an event please open an issue and we'll address it ASAP.

```rust
// This is an excerpt from crates/virtual-dom-rs/tests/events.
// To see more example event usage go to that file.
//
// Or better yet take a look at the web_sys API:
//   https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MouseEvent.html

{{#include ../../../crates/virtual-dom-rs/tests/events.rs:15:50}}
```

### Custom Event Handlers

Custom event handlers begin with a `!` and, like attributes must end with a `,`.

Percy will attach event handlers your DOM nodes via `addEventListener`

So `!mycustomevent` becomes `element.addEventListener('mycustomevent', callback)`

```rust
pub fn render (state: Rc<State>) -> VirtualNode {
  let state = Rc::clone(&self.state);

  let view = html! {
      <button
        !mycustomevent=move|| {
          state.borrow_mut().msg(Msg::ShowAlert)
        },>
        { "Dispatch 'mycustomevent' to me and I will do something!" }
     </button>
  };

  view
}
```

### Nested components

`html!` calls can be nested.

```rust
let view1 = html!{ <em> </em> };
let view2 = html{ <span> </span> }

let parent_view = html! {
  <div>
    { view1 }
    { view2 }
    html! {
      {"Nested html! call"}
    }
  </div>
};


let html_string = parent_view.to_string();
// Here's what the String looks like:
// <div><em></em><span></span>Nested html! call</div>
```
