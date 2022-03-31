use crate::app::APP;
use crate::fiber::FiberId;
use crate::Hook;
use crate::HookBuilder;
use crate::HookContext;
use std::any::Any;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

/// The type returned from a [use_state] hook
pub type State<T> = (Rc<T>, Rc<dyn Fn(T)>);

pub struct UseStateBuilder<T> {
    initial_value: T,
}

/// This hook is used to manage the state in a component.
///
/// Returns a [State<T>]
///
/// # Example
/// ```rust
/// #[component(Example)]
/// pub fn example() -> VNode {
///     let (counter, set_counter) = use_state(0);
///     let on_click = {
///         let counter = counter.clone();
///         let set_counter = set_counter.clone();
///         move || {
///             set_counter(*counter + 1);
///         }
///     };
///     Div.with_children(children!(
///         Button
///             .with_event(OnClick, callback!(move || on_click()))
///             .with_child("Increment value"),
///         P.with_child(format!("Current value {}", counter))
///     )).into()
/// }
/// ```
pub fn use_state<T: Any + Debug>(initial_value: T) -> UseStateBuilder<T> {
    UseStateBuilder { initial_value }
}

type StateHook<T> = Rc<RefCell<Rc<T>>>;

impl<T: Any + Debug> Hook for StateHook<T> {
    fn post_render(&mut self) {}
}

impl<T: Any + Debug> HookBuilder<State<T>> for UseStateBuilder<T> {
    fn build(self, (fiber_id, hook_context): &mut (FiberId, &mut HookContext)) -> State<T> {
        let (fiber_target_id, hook): (FiberId, &StateHook<T>) = {
            let hook_position = hook_context.counter;
            hook_context.counter += 1;
            if hook_position >= hook_context.hooks.len() {
                let initial_value = StateHook::new(RefCell::new(Rc::new(self.initial_value)));
                hook_context.hooks.push(Box::new(initial_value));
            }

            (*fiber_id, hook_context.get_mut_hook(hook_position))
        };
        let update_hook = hook.clone();
        let updater = move |new_value: T| {
            {
                update_hook.replace(Rc::new(new_value));
            }
            APP.with(|app| {
                if let Ok(mut app) = app.try_borrow_mut() {
                    if let Some(app) = &mut *app {
                        app.wip_root = Some(fiber_target_id);
                        app.next_unit_of_work = Some(fiber_target_id);
                    }
                }
            });
        };
        let hook = hook.borrow();
        (hook.clone(), Rc::new(updater))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_state() {
        let mut context = HookContext::default();

        {
            let hook_context = &mut (0, &mut context);

            let (state, set_state) = use_state(7).build(hook_context);

            assert_eq!(state, Rc::new(7));

            set_state(9);
        }

        context.counter = 0;
        let hook_context = &mut (0, &mut context);

        let (state, _) = use_state(7).build(hook_context);

        assert_eq!(state, Rc::new(9));
    }

    #[derive(Debug, PartialEq)]
    struct Test {
        i: u32,
        f: f32,
    }

    #[test]
    fn multiple_state() {
        let mut context = HookContext::default();

        {
            let hook_context = &mut (0, &mut context);

            let (int_state, set_int_state) = use_state(7).build(hook_context);
            let (string_state, set_string_state) = use_state("test".to_owned()).build(hook_context);
            let (struct_state, set_struct_state) =
                use_state(Test { i: 9, f: 3.4 }).build(hook_context);

            assert_eq!(int_state, Rc::new(7));
            assert_eq!(string_state, Rc::new("test".to_owned()));
            assert_eq!(struct_state, Rc::new(Test { i: 9, f: 3.4 }));

            set_int_state(9);
            set_string_state("test 2".to_owned());
            set_struct_state(Test { i: 1, f: 6.4 });
        }

        context.counter = 0;
        let hook_context = &mut (0, &mut context);

        let (int_state, _) = use_state(7).build(hook_context);
        let (string_state, _) = use_state("test".to_owned()).build(hook_context);
        let (struct_state, _) = use_state(Test { i: 9, f: 3.4 }).build(hook_context);

        assert_eq!(int_state, Rc::new(9));
        assert_eq!(string_state, Rc::new("test 2".to_owned()));
        assert_eq!(struct_state, Rc::new(Test { i: 1, f: 6.4 }));
    }
}
