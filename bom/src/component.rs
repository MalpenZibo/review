use crate::VNode;
use std::any::Any;
use std::any::TypeId;
use std::fmt::Debug;

pub trait ComponentProvider: Debug {
    type Props: Any;

    fn run(props: &Self::Props) -> VNode;

    fn get_props<'a>(&'a self) -> &'a Self::Props;
}

pub trait AnyComponent: Debug {
    fn run(&self) -> VNode;

    fn get_type(&self) -> TypeId;
}

impl<T: Any + ComponentProvider> AnyComponent for T {
    fn run(&self) -> VNode {
        T::run(self.get_props())
    }

    fn get_type(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

impl std::cmp::PartialEq<Box<dyn AnyComponent>> for Box<dyn AnyComponent> {
    fn eq(&self, other: &Box<dyn AnyComponent>) -> bool {
        self.get_type() == other.get_type()
    }
}

impl<T: Any + ComponentProvider> From<T> for VNode {
    fn from(v: T) -> VNode {
        VNode::Component(Box::new(v))
    }
}
