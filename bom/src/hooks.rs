use crate::app::APP;
use scoped_tls_hkt::scoped_thread_local;
use std::any::Any;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Debug)]
pub(crate) struct HookContext {
    pub hooks: Vec<Rc<RefCell<dyn Any>>>,
    pub counter: usize,
}

scoped_thread_local! {
    pub(crate) static mut HOOK_CONTEXT: HookContext
}

pub struct State<T, S: Fn(T) -> ()> {
    pub value: Rc<T>,
    set: S,
}

impl<T, S: Fn(T) -> ()> State<T, S> {
    pub fn set(&self, new_value: T) {
        (self.set)(new_value);
    }
}

pub fn use_state<T: Any + Debug>(initial_value: T) -> Rc<State<T, impl Fn(T)>> {
    let hook: Rc<RefCell<dyn Any>> = HOOK_CONTEXT.with(|mut hook_context| {
        let hook_position = hook_context.counter;
        hook_context.counter += 1;

        if hook_position >= hook_context.hooks.len() {
            let initial_value = Rc::new(RefCell::new(Rc::new(initial_value)));
            hook_context.hooks.push(initial_value.clone());
        }
        let cur_value = hook_context
            .hooks
            .get(hook_position)
            .expect("Retrieving hook error. Remember hook cannot be called conditionally")
            .clone();

        cur_value
    });

    let update_hook = hook.clone();

    let updater = move |new_value: T| {
        {
            let mut hook = update_hook.borrow_mut();
            let hook: &mut Rc<T> = hook
                .downcast_mut::<Rc<T>>()
                .expect("Incompatible hook type. Hooks must always be called in the same order");
            *hook = Rc::new(new_value);
        }

        APP.with(|app| {
            if let Ok(mut app) = app.try_borrow_mut() {
                if let Some(app) = &mut *app {
                    if app.working_context.working_complete() {
                        app.start_new_work();
                    }
                }
            }
        });
    };

    let hook = hook.borrow();
    let hook: &Rc<T> = hook
        .downcast_ref::<Rc<T>>()
        .expect("Incompatible hook type. Hooks must always be called in the same order");

    Rc::new(State {
        value: Rc::clone(&hook),
        set: updater,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_state() {
        let mut context = HookContext {
            hooks: Vec::default(),
            counter: 0,
        };

        HOOK_CONTEXT.set(&mut context, || {
            let state = use_state(7);

            assert_eq!(state.value, Rc::new(7));

            state.set(9);
        });

        context.counter = 0;

        HOOK_CONTEXT.set(&mut context, || {
            let state = use_state(7);

            assert_eq!(state.value, Rc::new(9));
        })
    }

    #[derive(Debug, PartialEq)]
    struct Test {
        i: u32,
        f: f32,
    }

    #[test]
    fn multiple_state() {
        let mut context = HookContext {
            hooks: Vec::default(),
            counter: 0,
        };

        HOOK_CONTEXT.set(&mut context, || {
            let int_state = use_state(7);
            let string_state = use_state("test".to_owned());
            let struct_state = use_state(Test { i: 9, f: 3.4 });

            assert_eq!(int_state.value, Rc::new(7));
            assert_eq!(string_state.value, Rc::new("test".to_owned()));
            assert_eq!(struct_state.value, Rc::new(Test { i: 9, f: 3.4 }));

            int_state.set(9);
            string_state.set("test 2".to_owned());
            struct_state.set(Test { i: 1, f: 6.4 });
        });

        context.counter = 0;

        HOOK_CONTEXT.set(&mut context, || {
            let int_state = use_state(7);
            let string_state = use_state("test".to_owned());
            let struct_state = use_state(Test { i: 9, f: 3.4 });

            assert_eq!(int_state.value, Rc::new(9));
            assert_eq!(string_state.value, Rc::new("test 2".to_owned()));
            assert_eq!(struct_state.value, Rc::new(Test { i: 1, f: 6.4 }));
        })
    }
}
