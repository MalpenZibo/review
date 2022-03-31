# Build a sample app

## Create a project using the default template

To get started, if you haven't already done it, install [cargo generate](https://github.com/cargo-generate/cargo-generate).
```bash
cargo install cargo-generate
```

Now you run 

```bash
cargo generate --git https://github.com/malpenzibo/review-template
```

go through the wizard specifing the project name and than enter in the new created folder and run

```bash
trunk serve
```

If everithing goes well you can see the result on [localhost:8080](http://localhost:8080) 

## Create a project from scratch

To get started, create a new project running 

```bash
cargo new review-app
```

and open the newly created directory.

```bash 
cd review-app
```

Now, update `Cargo.toml` adding `reView` as dependencies.

```toml
[package]
name = "review-app"
version = "0.1.0"
edition = "2021"

[dependencies]
review = "0.1.0"
```

### Update main.rs

We need to generate a component called `App` which renders a button that updates it's value when clicked. 

Replace the contents of `src/main.rs` with the following code.

```rust,noplayground
use review::Tag::{Button, Div};
use review::EventType::OnClick;
use review::{callback, children, component, use_state, ElementBuilder};

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

fn main() {
    review::init_logger(review::log::Level::Debug);

    review::render(App(()).into(), "root");
}
```

> NOTE
>
> The line `review::init_logger(review::log::Level::Debug);` will initialize the log system with `Debug` severity as minimum level.
> 
> The line `review::render(App(()).into(), "root");` inside `main()` starts your application and mounts it inside the element with the "root" id.
> 

### Create index.html

Finally, add an `index.html` file in the root directory of your app.

```html
<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8" />
    <title>reView App</title>
  </head>
  <body>
    <div id="root"></div>
  </body>
</html>
```

### View your web application

Run the following command to build and serve the application locally.

`trunk serve`

Trunk will helpfully rebuild your application if you modify any of its files.

### Congratulations

You have now successfully setup your reView development environment, and built your first web application and could see the result here [localhost:8080](http://localhost:8080) 