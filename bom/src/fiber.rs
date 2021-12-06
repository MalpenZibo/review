use crate::node::{Element, Node};
use crate::{Events, VNode};
use std::collections::HashMap;

pub(crate) type FiberId = usize;

#[derive(Debug)]
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
}

#[derive(Debug)]
pub(crate) enum EffectTag {
    Placement,
    Deletion,
    Update(UpdateData),
}

impl std::cmp::PartialEq<VNode> for FiberNode {
    fn eq(&self, other: &VNode) -> bool {
        match (&self.node, other) {
            (
                Node::Element(Element { tag: node_tag, .. }),
                VNode::Element { tag: vnode_tag, .. },
            ) => node_tag == vnode_tag,
            (Node::Text(_), VNode::Text(_)) => true,
            _ => false,
        }
    }
}

#[derive(PartialEq, Debug)]
pub(crate) enum State {
    Valid,
    Removed,
}

#[derive(Default, Debug)]
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
        } else {
            if let Some(old_child) = self
                .nodes
                .get_mut(parent_id)
                .and_then(|parent| parent.child.replace(child_id))
            {
                if let Some(child) = self.nodes.get_mut(child_id) {
                    child.sibling = Some(old_child)
                }
            }
        }

        if let Some(child) = self.nodes.get_mut(child_id) {
            child.parent = Some(parent_id);
        }
    }

    pub fn remove(&mut self, id: FiberId) {
        let reference = self.get(id).map(|node| (node.parent, node.sibling));
        if let Some((Some(parent_id), sibling_id)) = reference {
            if let Some(parent) = self.get_mut(parent_id) {
                parent.child = sibling_id
            }
        }

        if self.first_free_node.is_none() {
            self.first_free_node = Some(id);
        }

        let mut current_id = Some(id);
        while let Some(some_current_id) = current_id {
            if let Some(node) = self.get_mut(some_current_id) {
                node.state = State::Removed;

                if node.child.is_some() {
                    current_id = node.child;
                } else if node.sibling.is_some() {
                    current_id = node.sibling;
                } else if let Some(parent) = node.parent {
                    if parent != id {
                        current_id = self.get(parent).and_then(|parent_node| parent_node.sibling);
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
