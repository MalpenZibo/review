use crate::node::{Element, Node};
use crate::vdom::VElement;
use crate::AnyComponent;
use crate::{Events, VNode};
use std::collections::HashMap;
use std::rc::Rc;

#[doc(hidden)]
pub type FiberId = usize;

#[derive(Debug, PartialEq)]
pub(crate) struct FiberNode {
    pub node: Node,
    pub child: Option<FiberId>,
    pub parent: Option<FiberId>,
    pub sibling: Option<FiberId>,
    pub effect_tag: Option<EffectTag>,
    pub state: State,
}

#[derive(Debug)]
pub(crate) enum UpdateData {
    Text(String),
    Element {
        attributes: HashMap<String, String>,
        events: Events,
    },
    Component(Rc<dyn AnyComponent>),
}

#[derive(Debug, PartialEq)]
pub(crate) enum EffectTag {
    Placement,
    Deletion,
    Update(UpdateData),
}

impl PartialEq for UpdateData {
    fn eq(&self, other: &UpdateData) -> bool {
        match (self, other) {
            (UpdateData::Text(text_a), UpdateData::Text(text_b)) if text_a == text_b => true,
            (
                UpdateData::Element {
                    attributes: attributes_a,
                    events: events_a,
                },
                UpdateData::Element {
                    attributes: attributes_b,
                    events: events_b,
                },
            ) if attributes_a == attributes_b && events_a == events_b => true,
            (UpdateData::Component(component_a), UpdateData::Component(component_b))
                if Rc::ptr_eq(component_a, component_b) =>
            {
                true
            }
            _ => false,
        }
    }
}

impl std::cmp::PartialEq<VNode> for FiberNode {
    fn eq(&self, other: &VNode) -> bool {
        match (&self.node, other) {
            (
                Node::Element(Element { tag: node_tag, .. }),
                VNode::Element(VElement { tag: vnode_tag, .. }),
            ) => node_tag == vnode_tag,
            (Node::Text(_), VNode::Text(_)) => true,
            (Node::Component(_), VNode::Component(_)) => true,
            _ => false,
        }
    }
}

#[derive(PartialEq, Debug)]
pub(crate) enum State {
    Valid,
    Removed,
}

#[derive(Default, PartialEq, Debug)]
pub(crate) struct FiberTree {
    nodes: Vec<FiberNode>,
    first_free_node: Option<FiberId>,
}

impl FiberTree {
    pub fn new_node(&mut self, node: Node) -> FiberId {
        let new_fiber_node = FiberNode {
            child: None,
            sibling: None,
            parent: None,
            state: State::Valid,
            effect_tag: Some(EffectTag::Placement),
            node,
        };
        if let Some(first_free_node) = self.first_free_node {
            self.nodes[first_free_node] = new_fiber_node;
            let id = first_free_node;
            self.first_free_node = self.nodes.iter().position(|n| n.state == State::Removed);
            id
        } else {
            self.nodes.push(new_fiber_node);
            self.nodes.len() - 1
        }
    }

    pub fn get(&self, id: FiberId) -> Option<&FiberNode> {
        self.nodes.get(id)
    }

    pub fn get_mut(&mut self, id: FiberId) -> Option<&mut FiberNode> {
        self.nodes.get_mut(id)
    }

    pub fn insert_child(
        &mut self,
        child_id: FiberId,
        parent_id: FiberId,
        after_id: Option<FiberId>,
    ) {
        if let Some(after_id) = after_id {
            if let Some(old_sibling) = self
                .nodes
                .get_mut(after_id)
                .and_then(|after| after.sibling.replace(child_id))
            {
                if let Some(child) = self.nodes.get_mut(child_id) {
                    child.sibling = Some(old_sibling)
                }
            }
        } else if let Some(old_child) = self
            .nodes
            .get_mut(parent_id)
            .and_then(|parent| parent.child.replace(child_id))
        {
            if let Some(child) = self.nodes.get_mut(child_id) {
                child.sibling = Some(old_child)
            }
        }

        if let Some(child) = self.nodes.get_mut(child_id) {
            child.parent = Some(parent_id);
        }
    }

    pub fn remove(&mut self, id: FiberId) {
        let reference = self.get(id).map(|node| (node.parent, node.sibling));
        if let Some((Some(parent_id), sibling_id)) = reference {
            if let Some(first_child) = self.get(parent_id).and_then(|node| node.child) {
                if first_child == id {
                    if let Some(parent) = self.get_mut(parent_id) {
                        parent.child = sibling_id
                    }
                } else {
                    let mut prev_sibling = Some(first_child);
                    while let Some(some_prev_sibling) = prev_sibling {
                        let current = self.get(some_prev_sibling).and_then(|node| node.sibling);
                        if current == Some(id) {
                            break;
                        } else {
                            prev_sibling = current;
                        }
                    }
                    if let Some(prev_sibling) = prev_sibling {
                        if let Some(prev_sibling) = self.get_mut(prev_sibling) {
                            prev_sibling.sibling = sibling_id
                        }
                    }
                }
            }
        }

        if self.first_free_node.is_none() {
            self.first_free_node = Some(id);
        }

        if let Some(node) = self.get_mut(id) {
            node.state = State::Removed;
            let mut current_id = node.child;
            while let Some(some_current_id) = current_id {
                if let Some(node) = self.get_mut(some_current_id) {
                    node.state = State::Removed;

                    if node.child.is_some() {
                        current_id = node.child;
                    } else if node.sibling.is_some() {
                        current_id = node.sibling;
                    } else if let Some(parent) = node.parent {
                        if parent != id {
                            current_id =
                                self.get(parent).and_then(|parent_node| parent_node.sibling);
                        } else {
                            current_id = None;
                        }
                    } else {
                        current_id = None;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FiberTree;
    use crate::{
        fiber::{EffectTag, FiberNode, State},
        node::{Node, Text},
    };

    fn create_test_node(string: &str) -> Node {
        Node::Text(Text {
            dom: None,
            text: string.to_owned(),
        })
    }

    #[test]
    fn create_node() {
        let mut fiber_tree = FiberTree::default();

        fiber_tree.new_node(create_test_node("test"));

        assert_eq!(
            fiber_tree,
            FiberTree {
                nodes: vec!(FiberNode {
                    child: None,
                    sibling: None,
                    parent: None,
                    state: State::Valid,
                    effect_tag: Some(EffectTag::Placement),
                    node: Node::Text(Text {
                        dom: None,
                        text: "test".to_owned(),
                    })
                }),
                first_free_node: None
            }
        )
    }

    #[test]
    fn inser_child() {
        let mut fiber_tree = FiberTree::default();

        let parent_id = fiber_tree.new_node(create_test_node("test"));
        let child_id = fiber_tree.new_node(create_test_node("test child"));

        fiber_tree.insert_child(child_id, parent_id, None);

        assert_eq!(
            fiber_tree,
            FiberTree {
                nodes: vec!(
                    FiberNode {
                        child: Some(child_id),
                        sibling: None,
                        parent: None,
                        state: State::Valid,
                        effect_tag: Some(EffectTag::Placement),
                        node: Node::Text(Text {
                            dom: None,
                            text: "test".to_owned(),
                        })
                    },
                    FiberNode {
                        child: None,
                        sibling: None,
                        parent: Some(parent_id),
                        state: State::Valid,
                        effect_tag: Some(EffectTag::Placement),
                        node: Node::Text(Text {
                            dom: None,
                            text: "test child".to_owned(),
                        })
                    }
                ),
                first_free_node: None
            }
        )
    }

    #[test]
    fn inser_child_after_sibling() {
        let mut fiber_tree = FiberTree::default();

        let parent_id = fiber_tree.new_node(create_test_node("test"));
        let child_1_id = fiber_tree.new_node(create_test_node("test child 1"));
        let child_2_id = fiber_tree.new_node(create_test_node("test child 2"));

        fiber_tree.insert_child(child_1_id, parent_id, None);
        fiber_tree.insert_child(child_2_id, parent_id, Some(child_1_id));

        assert_eq!(
            fiber_tree,
            FiberTree {
                nodes: vec!(
                    FiberNode {
                        child: Some(child_1_id),
                        sibling: None,
                        parent: None,
                        state: State::Valid,
                        effect_tag: Some(EffectTag::Placement),
                        node: Node::Text(Text {
                            dom: None,
                            text: "test".to_owned(),
                        })
                    },
                    FiberNode {
                        child: None,
                        sibling: Some(child_2_id),
                        parent: Some(parent_id),
                        state: State::Valid,
                        effect_tag: Some(EffectTag::Placement),
                        node: Node::Text(Text {
                            dom: None,
                            text: "test child 1".to_owned(),
                        })
                    },
                    FiberNode {
                        child: None,
                        sibling: None,
                        parent: Some(parent_id),
                        state: State::Valid,
                        effect_tag: Some(EffectTag::Placement),
                        node: Node::Text(Text {
                            dom: None,
                            text: "test child 2".to_owned(),
                        })
                    }
                ),
                first_free_node: None
            }
        )
    }

    #[test]
    fn remove_node() {
        let mut fiber_tree = FiberTree::default();

        let parent_id = fiber_tree.new_node(create_test_node("test"));
        let child_1_id = fiber_tree.new_node(create_test_node("test child 1"));
        let child_2_id = fiber_tree.new_node(create_test_node("test child 2"));
        let child_3_id = fiber_tree.new_node(create_test_node("test child 3"));

        fiber_tree.insert_child(child_1_id, parent_id, None);
        fiber_tree.insert_child(child_2_id, parent_id, Some(child_1_id));
        fiber_tree.insert_child(child_3_id, parent_id, Some(child_2_id));

        fiber_tree.remove(child_1_id);

        assert_eq!(
            fiber_tree,
            FiberTree {
                nodes: vec!(
                    FiberNode {
                        child: Some(child_2_id),
                        sibling: None,
                        parent: None,
                        state: State::Valid,
                        effect_tag: Some(EffectTag::Placement),
                        node: Node::Text(Text {
                            dom: None,
                            text: "test".to_owned(),
                        })
                    },
                    FiberNode {
                        child: None,
                        sibling: Some(child_2_id),
                        parent: Some(parent_id),
                        state: State::Removed,
                        effect_tag: Some(EffectTag::Placement),
                        node: Node::Text(Text {
                            dom: None,
                            text: "test child 1".to_owned(),
                        })
                    },
                    FiberNode {
                        child: None,
                        sibling: Some(child_3_id),
                        parent: Some(parent_id),
                        state: State::Valid,
                        effect_tag: Some(EffectTag::Placement),
                        node: Node::Text(Text {
                            dom: None,
                            text: "test child 2".to_owned(),
                        })
                    },
                    FiberNode {
                        child: None,
                        sibling: None,
                        parent: Some(parent_id),
                        state: State::Valid,
                        effect_tag: Some(EffectTag::Placement),
                        node: Node::Text(Text {
                            dom: None,
                            text: "test child 3".to_owned(),
                        })
                    }
                ),
                first_free_node: Some(child_1_id)
            }
        )
    }

    #[test]
    fn get_node() {
        let mut fiber_tree = FiberTree::default();

        let parent_id = fiber_tree.new_node(create_test_node("test"));
        let child_id = fiber_tree.new_node(create_test_node("test child"));

        fiber_tree.insert_child(child_id, parent_id, None);

        assert_eq!(
            fiber_tree.get(parent_id),
            Some(&FiberNode {
                child: Some(child_id),
                sibling: None,
                parent: None,
                state: State::Valid,
                effect_tag: Some(EffectTag::Placement),
                node: Node::Text(Text {
                    dom: None,
                    text: "test".to_owned(),
                })
            })
        );
        assert_eq!(
            fiber_tree.get(child_id),
            Some(&FiberNode {
                child: None,
                sibling: None,
                parent: Some(parent_id),
                state: State::Valid,
                effect_tag: Some(EffectTag::Placement),
                node: Node::Text(Text {
                    dom: None,
                    text: "test child".to_owned(),
                })
            })
        );
    }

    #[test]
    fn get_first_child() {
        let mut fiber_tree = FiberTree::default();

        let parent_id = fiber_tree.new_node(create_test_node("test"));
        let child_id = fiber_tree.new_node(create_test_node("test child"));

        fiber_tree.insert_child(child_id, parent_id, None);
        let node = fiber_tree.get(parent_id);

        assert_eq!(
            fiber_tree.get(node.unwrap().child.unwrap()),
            Some(&FiberNode {
                child: None,
                sibling: None,
                parent: Some(parent_id),
                state: State::Valid,
                effect_tag: Some(EffectTag::Placement),
                node: Node::Text(Text {
                    dom: None,
                    text: "test child".to_owned(),
                })
            })
        );
    }

    #[test]
    fn get_sibling() {
        let mut fiber_tree = FiberTree::default();

        let parent_id = fiber_tree.new_node(create_test_node("test"));
        let child_id = fiber_tree.new_node(create_test_node("test child"));
        let sibling_id = fiber_tree.new_node(create_test_node("test sibling"));

        fiber_tree.insert_child(sibling_id, parent_id, None);
        fiber_tree.insert_child(child_id, parent_id, None);
        let node = fiber_tree.get(child_id);

        assert_eq!(
            fiber_tree.get(node.unwrap().sibling.unwrap()),
            Some(&FiberNode {
                child: None,
                sibling: None,
                parent: Some(parent_id),
                state: State::Valid,
                effect_tag: Some(EffectTag::Placement),
                node: Node::Text(Text {
                    dom: None,
                    text: "test sibling".to_owned(),
                })
            })
        );
    }

    #[test]
    fn get_parent() {
        let mut fiber_tree = FiberTree::default();

        let parent_id = fiber_tree.new_node(create_test_node("test"));
        let child_id = fiber_tree.new_node(create_test_node("test child"));

        fiber_tree.insert_child(child_id, parent_id, None);
        let node = fiber_tree.get(child_id);

        assert_eq!(
            fiber_tree.get(node.unwrap().parent.unwrap()),
            Some(&FiberNode {
                child: Some(child_id),
                sibling: None,
                parent: None,
                state: State::Valid,
                effect_tag: Some(EffectTag::Placement),
                node: Node::Text(Text {
                    dom: None,
                    text: "test".to_owned(),
                })
            })
        );
    }
}
