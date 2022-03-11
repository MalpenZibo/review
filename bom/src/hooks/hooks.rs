use crate::app::APP;
use crate::fiber::FiberId;
use std::any::Any;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct HookContext {
    pub hooks_state: Vec<Rc<RefCell<dyn Any>>>,
    pub counter: usize,
}

pub struct State<T, S: Fn(T)> {
    pub value: Rc<T>,
    set: S,
}

impl<T, S: Fn(T)> State<T, S> {
    pub fn set(&self, new_value: T) {
        (self.set)(new_value);
    }
}

pub fn use_state<T: Any + Debug>(
    initial_value: T,
    (fiber_id, hook_context): &mut (FiberId, &mut HookContext),
) -> Rc<State<T, impl Fn(T)>> {
    let (fiber_target_id, hook): (FiberId, Rc<RefCell<dyn Any>>) = {
        let hook_position = hook_context.counter;
        hook_context.counter += 1;

        if hook_position >= hook_context.hooks_state.len() {
            let initial_value = Rc::new(RefCell::new(Rc::new(initial_value)));
            hook_context.hooks_state.push(initial_value);
        }
        let cur_value = hook_context
            .hooks_state
            .get(hook_position)
            .expect("Retrieving hook error. Remember hook cannot be called conditionally")
            .clone();

        (*fiber_id, cur_value)
    };

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
                    app.wip_root = Some(fiber_target_id);
                    app.next_unit_of_work = Some(fiber_target_id);
                }
            }
        });
    };

    let hook = hook.borrow();
    let hook: &Rc<T> = hook
        .downcast_ref::<Rc<T>>()
        .expect("Incompatible hook type. Hooks must always be called in the same order");

    Rc::new(State {
        value: Rc::clone(hook),
        set: updater,
    })
}

