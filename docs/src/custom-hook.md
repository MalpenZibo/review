# Custom Hook

It's possible to define custom Hooks using multiple standard hook inside a single function.

For example if we want to log every state change we could create a custom hook like this:

```rust,noplayground
#[hook]
pub fn use_traced_state<T>(init_value: T) -> State<T>
where
    T: Any + PartialEq + Debug + Display,
{
    let (state, set_state) = use_state(init_value);

    use_effect(
        {
            let state = state.clone();
            move || {
                log::info!("{}", state);
                None::<fn()>
            }
        },
        Some(state.clone()),
    );

    (state, set_state)
}
```

In order to use the default hooks and become a custom hook a function should have the `hook` attribute.