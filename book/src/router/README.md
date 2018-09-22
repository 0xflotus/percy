# Router

We're working on tooling for view routing. Here's a preview of how it could look:

(Note that we're still thinking through the API so this is just a rough draft)

In the meantime definitely share your thoughts in the [Router tracking issue!](https://github.com/chinedufn/percy/issues/28)

```rust
#[macro_use]
extern crate virtual_dom_rs;

extern crate router_rs;
extern crate router_rs_macro;

use virtual_dom_rs::View;

use router_rs::{ViewRouter, Transition};
use router_rs_macro::router;

struct App {
    store: Rc<Store>,
}

struct Store {
    state: RefCell<State>
    vutils: ViewUtils,
    router: ViewRouter
}

impl Store {
    // ... still thinking ...
}

impl App {
    pub fn new () -> App {
        let mut router = ViewRouter::new();
        router
          .add_route(HomePage)
          .add_route(AuthorPage)
          .add_route(EditPostPage)
          .not_found(FourOhFourPage);
        let router = Rc::new(router);

        App {
            store: Store::new(router, vutils),
            vutils: ViewUtils::new(Rc::clone(&router)),
            router
        }
    }
}

#[route(path = "/")]
struct HomePage;

impl View for HomePage {
    fn render(
        &self,
        store: Rc<Store>,
    ) -> VirtualNode {
        html! {
            <div id='homepage',>
                <button !onclick || {
                    store.msg(Msg::Route("/posts/25/authors/jennifer"));
                },>
                    { "Get the behind the scenes on how" }
                    { " jenny helped write the latest post!" }
                </button>
            </div>
        }
    }
}

#[route(path = "/posts/{post_id}/authors/{name}")]
struct AuthorPage {
  post_id: u32,
  name: String
};

impl View for AuthorPage {
  fn render (
      &self,
      store: Rc<Store>,
  ) -> VirtualNode {
      match state.get_post(self.post_id()).get_author(&self.name()) {
          Some(ref author) => {
              html! {
                  { format!("Info about {}", author.name) }
              }
          }
          None => "Author does not exist"
      }
  }
}

struct IsAdmin;

impl BeforeEnteringRoute for IsAdmin {
    fn before_route (state: &State) -> Transition {
        if state.is_admin() {
            Transition::Continue
        } else {
            Transition::Redirect("/login")
        }
    }
}

#[route(path = "/posts/{post_id/edit", before = IsAdmin)]
struct EditPostPage {
    post_id: u32,
};

impl View for EditPostPage {
  fn render (
      &self,
      store: Rc<Store>,
  ) -> VirtualNode {
    html! { <div> { format!("Editing post {}", self.post_id())} </div> }
  }
}
```
