use crate::node::{Component, Element, Node, Text};
use crate::{AnyComponent, EventType, HookContext, Tag};
use std::collections::HashMap;
use std::iter::FromIterator;
use wasm_bindgen::JsValue;

#[derive(Debug, PartialEq)]
pub struct VElement {
    pub tag: Tag,
    pub attributes: HashMap<String, String>,
    pub events: Events,
    pub children: Vec<VNode>,
}

#[derive(Debug)]
pub enum VNode {
    Element(VElement),
    Text(String),
    Component(Box<dyn AnyComponent>),
}

impl std::cmp::PartialEq<VNode> for VNode {
    fn eq(&self, other: &VNode) -> bool {
        match (self, other) {
            (
                VNode::Element(VElement {
                    tag,
                    attributes,
                    events,
                    children,
                }),
                VNode::Element(VElement {
                    tag: other_tag,
                    attributes: other_attributes,
                    events: other_events,
                    children: other_children,
                }),
            ) => {
                tag == other_tag
                    && attributes == other_attributes
                    && events == other_events
                    && children == other_children
            }
            (VNode::Text(text), VNode::Text(other_text)) => text == other_text,
            (VNode::Component(component), VNode::Component(other_component)) => {
                component == other_component
            }
            _ => false,
        }
    }
}

impl VNode {
    pub(crate) fn materalize(self) -> Node {
        match self {
            VNode::Element(VElement {
                tag,
                attributes,
                events,
                children,
            }) => Node::Element(Element {
                tag,
                attributes,
                events,
                dom: None,
                unprocessed_children: children,
            }),
            VNode::Text(text) => Node::Text(Text { text, dom: None }),
            VNode::Component(component) => Node::Component(Component {
                hook_context: HookContext::default(),
                function: component,
            }),
        }
    }
}

pub type Event = Box<dyn AsRef<JsValue>>;

#[derive(Default)]
pub struct Events(pub HashMap<EventType, Event>);
impl std::fmt::Debug for Events {
    // Print out all of the event names for this VirtualNode
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let events: String = self.0.keys().map(|key| format!(" {:?}", key)).collect();
        write!(f, "{}", events)
    }
}

impl PartialEq for Events {
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }
        self.0.iter().all(|(key, value)| {
            other
                .0
                .get(key)
                .map_or(false, |v| value.as_ref().as_ref() == v.as_ref().as_ref())
        })
    }
}

impl From<&str> for VNode {
    fn from(v: &str) -> VNode {
        VNode::Text(v.to_owned())
    }
}

impl From<String> for VNode {
    fn from(v: String) -> VNode {
        VNode::Text(v)
    }
}

impl From<&String> for VNode {
    fn from(v: &String) -> VNode {
        VNode::Text(v.to_owned())
    }
}

impl From<Tag> for VNode {
    fn from(v: Tag) -> VNode {
        VNode::Element(VElement {
            tag: v,
            children: Vec::with_capacity(0),
            attributes: HashMap::with_capacity(0),
            events: Events(HashMap::with_capacity(0)),
        })
    }
}

impl From<VElement> for VNode {
    fn from(v: VElement) -> VNode {
        VNode::Element(v)
    }
}

pub trait ElementBuilder {
    fn with_child<T: Into<VNode>>(self, child: T) -> VElement;

    fn with_children(self, children: Vec<VNode>) -> VElement;

    fn with_attribute(self, key: &str, value: &str) -> VElement;

    fn with_attributes(self, attributes: Vec<(&str, &str)>) -> VElement;

    fn with_event(self, event: EventType, callback: Event) -> VElement;
}

impl ElementBuilder for VElement {
    fn with_attribute(mut self, key: &str, value: &str) -> VElement {
        self.attributes.insert(key.to_owned(), value.to_owned());

        self
    }

    fn with_attributes(mut self, attributes: Vec<(&str, &str)>) -> VElement {
        for (key, value) in attributes {
            self.attributes
                .insert((*key).to_owned(), (*value).to_owned());
        }

        self
    }

    fn with_child<T: Into<VNode>>(mut self, child: T) -> VElement {
        self.children.push(child.into());

        self
    }

    fn with_children(mut self, children: Vec<VNode>) -> VElement {
        for c in children {
            self.children.push(c)
        }

        self
    }

    fn with_event(mut self, event: EventType, callback: Event) -> VElement {
        self.events.0.insert(event, callback);

        self
    }
}

impl ElementBuilder for Tag {
    fn with_attribute(self, key: &str, value: &str) -> VElement {
        VElement {
            tag: self,
            attributes: HashMap::<String, String>::from([(key.to_owned(), value.to_owned())]),
            events: Events(HashMap::with_capacity(0)),
            children: Vec::with_capacity(0),
        }
    }

    fn with_attributes(self, attributes: Vec<(&str, &str)>) -> VElement {
        VElement {
            tag: self,
            attributes: HashMap::<String, String>::from_iter(
                attributes
                    .into_iter()
                    .map(|(key, value)| (key.to_owned(), value.to_owned())),
            ),
            events: Events(HashMap::with_capacity(0)),
            children: Vec::with_capacity(0),
        }
    }

    fn with_child<T: Into<VNode>>(self, child: T) -> VElement {
        VElement {
            tag: self,
            attributes: HashMap::with_capacity(0),
            events: Events(HashMap::with_capacity(0)),
            children: vec![child.into()],
        }
    }

    fn with_children(self, children: Vec<VNode>) -> VElement {
        let mut element = VElement {
            tag: self,
            attributes: HashMap::with_capacity(0),
            events: Events(HashMap::with_capacity(0)),
            children: Vec::with_capacity(children.len()),
        };
        for c in children {
            element.children.push(c);
        }

        element
    }

    fn with_event(self, event: EventType, callback: Event) -> VElement {
        VElement {
            tag: self,
            attributes: HashMap::with_capacity(0),
            events: Events(HashMap::<EventType, Event>::from([(event, callback)])),
            children: Vec::with_capacity(0),
        }
    }
}

#[macro_export]
macro_rules! callback {
    (|| $body:expr) => {
        ::std::boxed::Box::new(::review::Closure::wrap(
            ::std::boxed::Box::new(|| $body) as ::std::boxed::Box<dyn Fn()>
        ));
    };
    (move || $body:expr) => {
        ::std::boxed::Box::new(::review::Closure::wrap(
            ::std::boxed::Box::new(move || $body) as ::std::boxed::Box<dyn Fn()>,
        ));
    };
    (|$args:ident| $body:expr) => {
        ::std::boxed::Box::new(::review::Closure::wrap(
            ::std::boxed::Box::new(|$args| $body) as ::std::boxed::Box<dyn Fn(_)>,
        ));
    };
    (move |$args:ident| $body:expr) => {
        ::std::boxed::Box::new(::review::Closure::wrap(
            ::std::boxed::Box::new(move |$args| $body) as ::std::boxed::Box<dyn Fn(_)>,
        ));
    };
    (|$args:ident : $args_type:ty | $body:expr) => {
        ::std::boxed::Box::new(::review::Closure::wrap(::std::boxed::Box::new(
            |$args: $args_type| $body,
        )
            as ::std::boxed::Box<dyn Fn(_)>));
    };
    (move |$args:ident : $args_type:ty| $body:expr) => {
        ::std::boxed::Box::new(::review::Closure::wrap(::std::boxed::Box::new(
            move |$args: $args_type| $body,
        )
            as ::std::boxed::Box<dyn Fn(_)>));
    };
}

#[macro_export]
macro_rules! attributes {
    ( $( ($key:expr, $value:expr) ),* ) => {
        vec!($( ($key, &format!("{}", $value)) ),*)
    };
}

#[macro_export]
macro_rules! children {
    ( $( $child:expr ),* ) => {
        vec!($( $child.into() ),*)
    };
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::Events;
    use crate::Tag::Div;
    use crate::VElement;
    use crate::VNode;
    use std::collections::HashMap;

    #[test]
    fn create_element() {
        let div: VNode = Div.with_child("test").into();

        assert_eq!(
            div,
            VNode::Element(VElement {
                tag: Div,
                attributes: HashMap::default(),
                events: Events(HashMap::default()),
                children: vec!("test".into())
            })
        );
    }

    #[test]
    fn create_element_with_attribute() {
        let div: VNode = Div.with_attribute("name", "test").into();

        assert_eq!(
            div,
            VNode::Element(VElement {
                tag: Div,
                attributes: HashMap::<String, String>::from_iter(vec!((
                    "name".to_owned(),
                    "test".to_owned()
                )),),
                events: Events(HashMap::default()),
                children: Vec::default()
            })
        );
    }

    #[derive(Debug, PartialEq)]
    pub struct Test {
        cua: u32,
    }

    // #[component(Component1)]
    // fn component(props: &Test) -> VNode {
    //     format!("{}", props.cua).into()
    // }

    // #[test]
    // fn create_component() {
    //     let component: VNode = Component1(Test { cua: 8 }).into();

    //     assert_eq!(
    //         component,
    //         VNode::Component(Box::new(Component1(Test { cua: 8 })))
    //     );
    // }

    #[test]
    fn create_complex_vdom() {
        let vdom: VNode = Div
            .with_attributes(attributes!(("test", 5), ("test2", "7")))
            .with_children(children!("test", Div.with_child("test2")))
            .into();

        assert_eq!(
            vdom,
            VNode::Element(VElement {
                tag: Div,
                attributes: HashMap::<String, String>::from_iter(vec!(
                    ("test".to_owned(), "5".to_owned()),
                    ("test2".to_owned(), "7".to_owned())
                ),),
                events: Events::default(),
                children: vec!(
                    VNode::Text("test".to_owned()),
                    VNode::Element(VElement {
                        tag: Div,
                        attributes: HashMap::default(),
                        events: Events::default(),
                        children: vec!(VNode::Text("test2".to_owned()))
                    })
                )
            })
        );
    }
}
