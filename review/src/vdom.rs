use crate::node::{Component, Element, Node, Text};
use crate::{AnyComponent, EventType, HookContext, Tag};
use std::collections::HashMap;
use std::iter::FromIterator;
use wasm_bindgen::JsValue;

/// A VElement is a particulat type of [VNode] generated from a [Tag]
#[derive(Debug, PartialEq)]
pub struct VElement {
    pub tag: Tag,
    pub attributes: HashMap<String, String>,
    pub events: Events,
    pub children: Vec<VNode>,
}

/// A VNode rappresent a node in the VirtualDOM tree.
///
/// It could be a [VElement] obtained from a [Tag],
/// a simple text node obtained from a [String]
/// or a Component node obtained from a function component
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

#[doc(hidden)]
pub type Event = Box<dyn AsRef<JsValue>>;

#[doc(hidden)]
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

/// This is the API to declare [VNode] in reView
///
/// reView doesn't implement any macro to mimic JSX syntax (too much effort to develop, test, and maintain such macro)
///
/// Instead it define a fluent API to declare [VNode] from a [Tag] or a [VElement].
/// Then the obtained [VElement] can be converted in a [VNode] calling into()
/// # Example
/// ```rust
/// Main.with_children(children!(
///  Img.with_attribute("class", "logo")
///         .with_attribute("src", "/assets/logo.png")
///         .with_attribute("alt", "reView logo"),
///      H1.with_child("Hello World!"),
///      Span.with_attribute("class", "subtitle")
///          .with_children(children!(
///              "from reView with ",
///              I.with_attribute("class", "heart")
///          ))
///      ))
///      .into()
///```
pub trait ElementBuilder {
    /// This function is used to append a child that implements [Into<VNode>] to a [Tag] or a [VElement] and return a [VElement]
    ///
    /// # Example
    /// ```rust
    /// let mut velement = Div.with_child(Button);
    /// velement.with_child(Span.with_child("hello!!"));
    /// ```
    fn with_child<T: Into<VNode>>(self, child: T) -> VElement;

    /// This function is used to append a list of children [VNode] to a [Tag] or a [VElement] and return a [VElement]
    ///
    /// # Example
    /// ```rust
    /// let velement = Div.with_children(vec!(Button.into(), Div.into(), A.into()));
    /// ```
    /// or to avoid the `.into()` calls
    /// # Example
    /// ```rust
    /// let velement = Div.with_children(children!(Button, Div, A));
    /// ```
    fn with_children(self, children: Vec<VNode>) -> VElement;

    /// This function is used to append an attribute to a [Tag] or a [VElement] and return a [VElement]
    ///
    /// # Example
    /// ```rust
    /// let mut velement = A.with_attribute("href", "https://malpenzibo.github.io/review/");
    /// velement.with_attribute("target", "_blank");
    /// ```
    fn with_attribute(self, key: &str, value: &str) -> VElement;

    /// This function is used to append a list of attributes to a [Tag] or a [VElement] and return a [VElement]
    ///
    /// # Example
    /// ```rust
    /// let velement = Div.with_attributes(vec!(
    ///     ("href", "https://malpenzibo.github.io/review/"),
    ///     ("target", "_blank")
    /// ));
    /// ```
    fn with_attributes(self, attributes: Vec<(&str, &str)>) -> VElement;

    /// This function is used to append an event to a [Tag] or a [VElement] and return a [VElement]
    ///
    /// # Example
    /// ```rust
    /// let mut velement = Div.with_event(OnClick, callback!(move || log::info!("hello!!")));
    /// velement.with_event(OnMouseEnter, callback!(move || log::info!("mouseEnter!!")));
    /// ```
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

/// This macro is used to declare event for a [Tag] element using the [ElementBuilder] API
///
/// # Example
/// ```rust
/// Button.with_event(OnClick, callback!(move || log::info!("hello from js on click event")))
/// ```
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

/// This macro helps to declare children for a [Tag] element using the [ElementBuilder] API
///
/// # Example
/// ```rust
/// Div.with_children(children!("test", Div.with_child("test2")))
/// ```
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
    use crate::Tag::*;
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
            .with_attributes(vec![("test", "5"), ("test2", "7")])
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
