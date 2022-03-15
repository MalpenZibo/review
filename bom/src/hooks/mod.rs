use crate::fiber::FiberId;
use std::fmt::Debug;

mod use_effect;
mod use_state;

use downcast_rs::{impl_downcast, Downcast};
pub use use_effect::use_effect;
pub use use_state::use_state;

#[derive(Debug, Default)]
pub struct HookContext {
    pub hooks: Vec<Box<dyn Hook>>,
    pub counter: usize,
}

impl HookContext {
    fn get_mut_hook<T: Hook>(&mut self, hook_position: usize) -> &mut T {
        self.hooks
            .get_mut(hook_position)
            .and_then(|hook| hook.downcast_mut::<T>())
            .expect("Hook retrieval error")
    }
}

pub trait HookBuilder<T> {
    fn build(self, context: &mut (FiberId, &mut HookContext)) -> T;
}

pub trait Hook: Downcast + Debug {
    fn post_render(&mut self);
}
impl_downcast!(Hook);
