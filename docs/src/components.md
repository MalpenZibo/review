# Components

A Component consist of a single function that receives props and determines what should be rendered by returning a `VNode`. 
Without `hooks` a component is quite limiting because can only be a pure component. 
`Hooks` allow components to maintain their own internal state and use other reView features.

## Creating a Components

A component can be created using the `#[component]` attribute on top of a function.

```rust,noplayground
#[component(ExampleComponent)]
pub fn example_component() -> VNode {
    Div.into()
}
```

## Under the hood

A functional component is a struct that implements the `ComponentProvider` trait. This trait has two methods, the `render` method and the `get_props` method.
The first one is used to retrieve the `VNode` produced by the component and the second one il used to get a reference to the props that should be send to the `render` method.

The `component` attribute will automatically create a struct that implement the `ComponentProvider` using the specified function. 
Also checks that the fuction respect the hook rules.