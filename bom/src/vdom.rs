use crate::node::{Component, Element, Node, Text};
use crate::{AnyComponent, EventType, HookContext, Tag};
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::JsValue;

#[derive(Debug)]
pub enum VNode {
    Element {
        tag: Tag,
        attributes: HashMap<String, String>,
        events: Events,
        children: Vec<VNode>,
    },
    Text(String),
    Component(Box<dyn AnyComponent>),
}

impl VNode {
    pub(crate) fn to_node(self) -> Node {
        match self {
            VNode::Element {
                tag,
                attributes,
                events,
                children,
            } => Node::Element(Element {
                tag,
                attributes,
                events,
                dom: None,
                unprocessed_children: children,
            }),
            VNode::Text(text) => Node::Text(Text { text, dom: None }),
            VNode::Component(component) => Node::Component(Component {
                hooks: HookContext {
                    hooks: Vec::default(),
                    counter: 0,
                },
                function: component,
            }),
        }
    }
}

#[derive(Debug)]
pub struct ElementBuilder {
    tag: Tag,
    attributes: HashMap<String, String>,
    events: Events,
    children: Vec<VNode>,
}
pub type DynClosure = Rc<dyn AsRef<JsValue>>;

#[derive(Clone)]
pub struct Events(pub HashMap<EventType, DynClosure>);
impl std::fmt::Debug for Events {
    // Print out all of the event names for this VirtualNode
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let events: String = self
            .0
            .keys()
            .map(|key| format!(" {:?}", key).to_string())
            .collect();
        write!(f, "{}", events)
    }
}

impl PartialEq for Events {
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }
        self.0
            .iter()
            .all(|(key, value)| other.0.get(key).map_or(false, |v| Rc::ptr_eq(value, v)))
    }
}

pub(crate) struct EventsVec(pub Vec<(EventType, DynClosure)>);
impl std::fmt::Debug for EventsVec {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let events: String = self
            .0
            .iter()
            .map(|v| format!(" {:?}", v.0).to_string())
            .collect();
        write!(f, "{}", events)
    }
}

impl ElementBuilder {
    pub fn with_attribute(mut self, key: &str, value: &str) -> Self {
        self.attributes.insert(key.to_owned(), value.to_owned());

        self
    }

    pub fn with_attributes(mut self, attributes: &[(&str, &str)]) -> Self {
        for (key, value) in attributes {
            self.attributes
                .insert((*key).to_owned(), (*value).to_owned());
        }

        self
    }

    pub fn with_child(mut self, child: VNode) -> Self {
        self.children.push(child);

        self
    }

    pub fn with_event(mut self, event: EventType, callback: DynClosure) -> Self {
        self.events.0.insert(event, callback);

        self
    }

    pub fn build(self) -> VNode {
        VNode::Element {
            tag: self.tag,
            attributes: self.attributes,
            events: self.events,
            children: self.children,
        }
    }
}

pub fn create_element(tag: Tag) -> ElementBuilder {
    ElementBuilder {
        tag,
        attributes: HashMap::default(),
        events: Events(HashMap::default()),
        children: Vec::default(),
    }
}

pub fn create_text(text: &str) -> VNode {
    VNode::Text(text.to_owned())
}

#[macro_export]
macro_rules! callback {
    (|| $body:expr) => {
        std::rc::Rc::new(Closure::wrap(Box::new(|| $body) as Box<dyn Fn()>));
    };
    (move || $body:expr) => {
        std::rc::Rc::new(Closure::wrap(Box::new(move || $body) as Box<dyn Fn()>));
    };
    (|$args:ident| $body:expr) => {
        std::rc::Rc::new(Closure::wrap(Box::new(|$args| $body) as Box<dyn Fn(_)>));
    };
    (move |$args:ident| $body:expr) => {
        std::rc::Rc::new(Closure::wrap(Box::new(move |$args| $body) as Box<dyn Fn(_)>));
    };
    (|$args:ident : $args_type:ty | $body:expr) => {
        std::rc::Rc::new(Closure::wrap(
            Box::new(|$args: $args_type| $body) as Box<dyn Fn(_)>
        ));
    };
    (move |$args:ident : $args_type:ty| $body:expr) => {
        std::rc::Rc::new(Closure::wrap(
            Box::new(move |$args: $args_type| $body) as Box<dyn Fn(_)>
        ));
    };
}
