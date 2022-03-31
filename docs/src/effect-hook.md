# Effect Hook

`use_effect` is used for hooking into the component's lifecycle and creating side-effects.

It takes a function which is called every time after the component's render has finished.

This function returns an Optional closure that will be called after the execution of the effect. 
This optional closure is a cleanup function and it's usefull if is necessary to perform some cleanups after the execution of the effect.

The `use_effect` hooks accept, as second optional argument, the effect dependencies. 
Only when the dependencies change, it calls the provided function.

> Note
> 
> `dependencies` must implement `PartialEq`

## Examples

### Without cleanup and dependencies

```rust,noplayground
#[component(StateExample)]
fn state_example() -> VNode {
    let (state, set_state) = use_state(0);

    use_effect(
        || {
            review::log::info!("hello world!!");
            None::<fn()>
        },
        None::<()>,
    );

    Div.with_children(children!(
        format!("Current value {}", state),
        Button
            .with_child("Increase counter")
            .with_event(OnClick, callback!(move || { set_state(*state + 1) }))
    ))
    .into()
}
```

### With dependencies and without cleanup

```rust,noplayground
#[component(StateExample)]
fn state_example() -> VNode {
    let (state, set_state) = use_state(0);

    use_effect(
        || {
            review::log::info!("hello world!!");
            None::<fn()>
        },
        Some(*state),
    );

    Div.with_children(children!(
        format!("Current value {}", state),
        Button
            .with_child("Increase counter")
            .with_event(OnClick, callback!(move || { set_state(*state + 1) }))
    ))
    .into()
}
```

### With dependencies and cleanup

```rust,noplayground
#[component(StateExample)]
fn state_example() -> VNode {
    let (state, set_state) = use_state(0);

    use_effect(
        || {
            log::info!("hello world!!");
            Some(|| log::info!("cleanup"))
        },
        Some(*state),
    );

    Div.with_children(children!(
        format!("Current value {}", state),
        Button
            .with_child("Increase counter")
            .with_event(OnClick, callback!(move || { set_state(*state + 1) }))
    ))
    .into()
}
```