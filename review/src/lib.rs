//! # reView - API Documentation
//!
//! reView is a Rust library for creating front-end web apps using WebAssembly
//!
//! - Features a fluent API for declaring interactive HTML with Rust expressions.
//! - Use a VirtualDOM to minimize DOM API calls for each page render.
//!
//! > reView is not production-ready, and it's a WIP project so expect breaking changes between versions.
//!
//! ### Supported Targets (Client-Side Rendering)
//! - `wasm32-unknown-unknown`
//!
//! ## Example
//!
//! ```rust,no_run
//! use review::EventType::OnClick;
//! use review::Tag::{Button, Div};
//! use review::{callback, children, component, use_state, ElementBuilder, VNode};
//!
//! #[component(App)]
//! pub fn app() -> VNode {
//!     let (state, set_state) = use_state(0);
//!
//!     Div.with_children(children!(
//!         format!("Current value {}", state),
//!         Button
//!             .with_child("Increase counter")
//!             .with_event(OnClick, callback!(move || { set_state(*state + 1) }))
//!     ))
//!     .into()
//! }
//!
//! fn main() {
//!     review::init_logger(review::log::Level::Debug);
//!
//!     review::render(App(()).into(), "root");
//! }
//! ```

mod app;
mod component;
mod events;
mod fiber;
mod hooks;
mod node;
mod reconciliation;
mod tag;
mod utils;
mod vdom;

pub use app::*;
pub use component::*;
pub use events::*;
pub use fiber::FiberId;
pub use hooks::HookContext;
pub use hooks::*;
pub use tag::*;
pub use utils::*;
pub use vdom::*;

pub use log;

#[doc(hidden)]
pub use wasm_bindgen::closure::Closure;

extern crate review_macro;
/// This attribute creates a component from a normal Rust function.
///
/// Functions with this attribute must return a [VNode] and can optionally take an argument for props.
/// Note that the function only receives a reference to the props.
///
/// When using this attribute you need to provide a name for the component: `#[component(ComponentName)]`.
/// The attribute will then automatically create a Component with the given identifier which you can use like a normal struct.
///
/// # Example
/// ```rust
/// # use review::{component, ElementBuilder};
/// # use review::Tag::P;
/// # #[derive(Debug)]
/// # pub struct Props {};
/// #[component(Board)]
/// pub fn board(props: &Props) -> VNode {
///     P.with_child(format!("{:?}", props)).into()
/// }
/// ```
pub use review_macro::component;

/// This attribute creates a user-defined hook from a normal Rust function.
///
/// Function with this attribute must have a name starting with `use_`. Without using
/// this attribute the function could not use the predefined hooks.
///
/// # Example
/// ```rust
/// # use review::{hook, State, use_state, use_effect, log};
/// #[hook]
/// pub fn use_example() -> State<u32> {
///     let (c, set_c) = use_state(5);
///     use_effect(
///         {
///             let c = c.clone();
///             move || {
///                 log::info!("{}", c);
///
///                 None::<fn()>
///             }
///         },
///         Some(c.clone()),
///     );
///
///     (c, set_c)
/// }
/// ```
pub use review_macro::hook;
