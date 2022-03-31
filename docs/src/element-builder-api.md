# Element Builder API

reView doesn't have a macro to support a syntax similar to jsx but instead provides an API to create the UI.

The main concept is that every `Tag` and every `String` could become a `VNode`.

For this reason both `Tag`, `String` and `VElement` implement the `From<VNode>` trait to obtain a `VNode` using `into()`.

To attach attributes or children to a `Tag` element we could use a builder API.

Every `Tag` or `VElement` implements the `ElementBuilder` trait so we can call:
- `with_child` to attach a single child (the child should implement the `From<VNode>` trait)
- `with_children` to attach a vector of child (every element should implement). reView provide a `children!` macro to simplify the creation of the child vector
- `with_attribute` to attach an attribute specifing a`key` and a `value`
- `with_attributes` to attach a vector of attributes. A vector of attribute is a vector of tuple `(key, value)`
- `with_event` to attach an event specifing a `EventType` and an `Event`. reView provide a `callback!` macro to create an `Event` from a rust closure.

Using this API we can create a customized `VElement` that could be converted into a `VNode` with `into()`.

```rust,noplayground
Main.with_children(children!(
    Img.with_attribute("class", "logo")
        .with_attributes(vec!(
            ("src", "/assets/logo.png"), 
            ("alt", "reView logo")
        ))
        .with_event(OnClick, callback!(|| { log::info!("hello!!") }))
    H1.with_child("Hello World!"),
    Span.with_attribute("class", "subtitle")
        .with_children(children!(
            "from reView with ",
            I.with_attribute("class", "heart")
        ))
))
.into()
```