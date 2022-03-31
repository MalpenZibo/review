# State Hook

`use_state` is used to manage state in a component. 
It returns a `State<T>` that is a tuple with the first element that is a `Rc<T>` with the current stored values and as second element an `Rc<Fn(T)>`  to change the current stored values.

The hook takes a value as input which determines the initial value.

## Example

```rust,noplayground
#[component(StateExample)]
fn state_example() -> VNode {
    let (state, set_state) = use_state(0);

    Div.with_children(children!(
        format!("Current value {}", state),
        Button
            .with_child("Increase counter")
            .with_event(OnClick, callback!(move || { set_state(*state + 1) }))
    ))
    .into()
}
```
