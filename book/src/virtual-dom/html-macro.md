# Writing html!

### Static text

Text that will never change can be typed right into your HTML

```rust
use virtual_dom_rs::prelude::*;

html!{
  <div> Text goes here </div>
};
```

### Text variables

Text variables much be wrapped in the `text!` macro.

```rust
use virtual_dom_rs::prelude::*;

let text_var = " world"

html! {
  Hello { text!(text_var) }
}
```

### Attributes

Attributes work just like regular HTML.

```rust
let view = html!{
  <div id='my-id' class='big wide'></div>
};
```

### Event Handlers

```rust
html! {
    <button
      onclick=move|_event: web_sys::MouseEvent| {
        web_sys::console::log_1(&"clicked!".into());
      }
    >
      Click me!
    </button>
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
      Nested html! call
    }
  </div>
};


let html_string = parent_view.to_string();
// Here's what the String looks like:
// <div><em></em><span></span>Nested html! call</div>
```
