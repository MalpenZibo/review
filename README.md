# ReView

## About
ReView is a React-inspired library for didactic purposes written in Rust.

This project is inspired by a series of posts that explains how to build a React clone from scratch (https://github.com/pomber/didact). I liked the idea, so I tried to create a similar project using Rust.
In the process, I take inspiration for the component macro and the hook functionality from Yew (https://github.com/yewstack/yew).

## How it works
You could see the `review-example`. 

To create a project see: https://github.com/rustwasm/rust-webpack-template 
Then you could simply add review as a local dependecy

```toml
[dependencies]
review = { path = "../review" }
```

add a root div inside the `static/index.html`  files
```html
<body>
  <div id="root"></div>
  <script src="index.js"></script>
</body>
```

and in the end you could create your first component using ReView.

##### `lib.rs`
```rust
#[component(App)]
pub fn app() -> VNode {
    let (state, set_state) = use_state(0);

    Div.with_children(children!(
        format!("Current value {}", state),
        Button
            .with_child("Increase counter")
            .with_event(OnClick, callback!(move || { set_state(*state + 1) }))
    ))
    .into()
}

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    review::init_logger();

    review::render(App(()).into(), "root");

    Ok(())
}
```

That's it, a simple button that increments a counter :D

### ReView Example
I implemented a simple Tic Tac Toe game like in the standard React tutorial https://reactjs.org/tutorial/tutorial.html

Play `ReView Tic Tac Toe` here: https://malpenzibo.github.io/review/

![Tic Tac Toe](https://raw.githubusercontent.com/MalpenZibo/review/main/docs/tic%20tac%20toe.gif)