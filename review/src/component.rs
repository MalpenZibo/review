use crate::fiber::FiberId;
use crate::HookContext;
use crate::VNode;
use std::any::Any;
use std::any::TypeId;
use std::fmt::Debug;
use std::rc::Rc;

#[doc(hidden)]
pub trait ComponentProvider: Debug {
    type Props: Any;

    fn render(context: &mut (FiberId, &mut HookContext), props: &Self::Props) -> VNode;

    fn get_props(&self) -> &Self::Props;
}

#[doc(hidden)]
pub trait AnyComponent: Debug {
    fn render(&self, context: &mut (FiberId, &mut HookContext)) -> VNode;

    fn get_type(&self) -> TypeId;
}

impl<T: Any + ComponentProvider> AnyComponent for T {
    fn render(&self, context: &mut (FiberId, &mut HookContext)) -> VNode {
        context.1.counter = 0;
        let node = T::render(context, self.get_props());

        for h in context.1.hooks.iter_mut() {
            h.post_render();
        }

        node
    }

    fn get_type(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

impl std::cmp::PartialEq<dyn AnyComponent> for dyn AnyComponent {
    fn eq(&self, other: &dyn AnyComponent) -> bool {
        self.get_type() == other.get_type()
    }
}

impl<T: Any + ComponentProvider> From<T> for VNode {
    fn from(v: T) -> VNode {
        VNode::Component(Rc::new(v))
    }
}
