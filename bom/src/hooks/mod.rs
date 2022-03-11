use crate::fiber::FiberId;
use std::any::Any;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

mod use_state;

pub use use_state::use_state;
#[derive(Debug, Default)]
pub struct HookContext {
    pub hooks_state: Vec<Rc<RefCell<dyn Any>>>,
    pub counter: usize,
}

pub trait HookBuilder<T> {
    fn build(self, context: &mut (FiberId, &mut HookContext)) -> T;
}
