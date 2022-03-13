use crate::fiber::FiberId;
use std::any::Any;
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

pub trait HookBuilder<T> {
    fn build(self, context: &mut (FiberId, &mut HookContext)) -> T;
}

pub trait Hook: Downcast + Debug {
    fn pre_render(&mut self);

    fn post_render(&mut self);
}
impl_downcast!(Hook);
