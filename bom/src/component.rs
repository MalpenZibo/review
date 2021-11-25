use crate::VNode;
use std::any::Any;
use std::any::TypeId;
use std::fmt::Debug;
use std::marker::PhantomData;

pub trait ComponentProvider: Debug {
    type Props: Any + PartialEq + Debug;

    fn run(props: &Self::Props) -> VNode;
}

#[derive(Debug)]
pub struct Component<T: ComponentProvider> {
    _never: PhantomData<T>,
    props: T::Props,
}

pub trait AnyComponent: Debug {
    fn run(&self) -> VNode;

    fn get_type(&self) -> TypeId;
}

impl<T: Any + ComponentProvider> AnyComponent for Component<T> {
    fn run(&self) -> VNode {
        T::run(&self.props)
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

pub fn create_component<T: ComponentProvider + 'static>(props: T::Props) -> VNode {
    VNode::Component(Box::new(Component::<T> {
        _never: std::marker::PhantomData::default(),
        props,
    }))
}
