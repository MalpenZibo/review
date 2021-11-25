use crate::component::AnyComponent;
use crate::create_element_dom;
use crate::create_text_dom;
use crate::hooks::HOOK_CONTEXT;
use crate::tag::Tag;
use crate::vdom::DynClosure;
use crate::vdom::VNode;
use crate::App;
use crate::EventType;
use crate::Events;
use crate::EventsVec;
use crate::HookContext;
use indextree::Arena;
use indextree::Node;
use indextree::NodeId;
use log::debug;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::JsCast;

#[derive(Debug)]
pub(crate) enum Patch {
    SetAttributes(NodeId, Vec<(String, String)>),
    DropAttributes(NodeId, Vec<String>),
    SetEvents(NodeId, EventsVec),
    UpdateEvents(NodeId, EventsVec),
    DropEvents(NodeId, EventsVec),
    AppendBefore(NodeId, NodeId),
    Append(NodeId),
    Drop(NodeId),
    DropFromToEnd(NodeId),
    DropChildren(NodeId),
}

#[derive(Debug)]
pub(crate) enum DomNode {
    Element {
        dom: Option<web_sys::Element>,
        tag: Tag,
        attributes: HashMap<String, String>,
        events: Events,
    },
    Text {
        dom: Option<web_sys::Text>,
        text: String,
    },
    Component {
        hooks: HookContext,
        function: Box<dyn AnyComponent>,
    },
}

impl std::cmp::PartialEq<VNode> for DomNode {
    fn eq(&self, other: &VNode) -> bool {
        match (self, other) {
            (DomNode::Element { tag: node_tag, .. }, VNode::Element { tag: vnode_tag, .. }) => {
                node_tag == vnode_tag
            }
            (
                DomNode::Text {
                    text: node_text, ..
                },
                VNode::Text(vnode_text),
            ) => node_text == vnode_text,
            (DomNode::Component { function, .. }, VNode::Component(component)) => {
                function == component
            }
            _ => false,
        }
    }
}

#[derive(Debug)]
pub(crate) struct WorkingContext {
    pub vnode_buffer: Vec<(usize, VNode)>,
    pub patches: Vec<Patch>,
    pub last_deep: usize,
    pub current_node_id: NodeId,
    pub last_parent: NodeId,
}

impl WorkingContext {
    pub fn working_complete(&self) -> bool {
        self.vnode_buffer.is_empty()
    }
}

pub(crate) fn work_on_vdom<F: Fn() -> bool>(app: &mut App, continue_working: F) {
    let App {
        node_tree,
        root,
        working_context,
        document,
    } = app;

    while !working_context.vnode_buffer.is_empty() && continue_working() {
        if let Some((deep, current_vnode)) = working_context.vnode_buffer.pop() {
            debug!("deep {:?}, last_deep {:?}", deep, working_context.last_deep);

            let working_node_id: Option<NodeId> = match (deep, working_context.last_deep) {
                (0, _) => Some(*root),
                (deep, last_deep) if deep == last_deep => {
                    debug!("move to sibling from {:?}", working_context.current_node_id);
                    node_tree
                        .get(working_context.current_node_id)
                        .unwrap()
                        .next_sibling()
                }
                (deep, last_deep) if deep > last_deep => {
                    working_context.last_parent = working_context.current_node_id;
                    debug!("move to child from {:?}", working_context.current_node_id);
                    node_tree
                        .get(working_context.current_node_id)
                        .and_then(|node| node.first_child())
                }
                (deep, last_deep) if deep < last_deep => working_context
                    .current_node_id
                    .ancestors(node_tree)
                    .skip(last_deep - deep)
                    .next()
                    .and_then(|parent_id| {
                        drop_remaining(
                            working_context.current_node_id,
                            parent_id,
                            node_tree,
                            &mut working_context.patches,
                        );
                        node_tree.get(parent_id)
                    })
                    .and_then(|parent| {
                        if let Some(parent) = parent.parent() {
                            working_context.last_parent = parent;
                        }
                        debug!("move to parent from {:?}", working_context.current_node_id);
                        parent.next_sibling()
                    }),
                _ => None,
            };
            debug!(
                "current_vnode {:?}, working_node_id {:?}, last_parent {:?}",
                current_vnode, working_node_id, working_context.last_parent
            );
            let (new_node, children) = reconcile(
                working_context.last_parent,
                working_node_id,
                Some(current_vnode),
                node_tree,
                &mut working_context.patches,
                document.as_ref(),
            );
            working_context.current_node_id = new_node;
            if let Some(children) = children {
                debug!("children {:?}", children);
                let next_deep = deep + 1;
                for c in children.into_iter().rev() {
                    working_context.vnode_buffer.push((next_deep, c));
                }
            }
            working_context.last_deep = deep;
        }
    }

    // current_node_id is not the root so because I'm inside the fiber tree
    // Drop the rest of the nodes
    if working_context.vnode_buffer.is_empty() && working_context.current_node_id != *root {
        drop_remaining(
            working_context.current_node_id,
            *root,
            node_tree,
            &mut working_context.patches,
        );
    }
}

fn find_parent<'a>(node: &'a Node<DomNode>, node_tree: &'a Arena<DomNode>) -> Option<&'a DomNode> {
    let mut parent_id = node.parent();

    while let Some(some_parent_id) = parent_id {
        if let Some(parent_node) = node_tree.get(some_parent_id) {
            match parent_node.get() {
                parent @ DomNode::Element { .. } => {
                    return Some(parent);
                }
                _ => {
                    parent_id = parent_node.parent();
                }
            }
        } else {
            return None;
        }
    }

    None
}

pub(crate) fn commit_patches(app: &mut App) {
    let App {
        node_tree,
        working_context,
        ..
    } = app;

    debug!("commit patches {:?}", working_context.patches);
    for p in working_context.patches.drain(..) {
        match p {
            Patch::SetAttributes(node_id, mut attributes_to_add) => {
                if let Some(DomNode::Element {
                    dom,
                    ref mut attributes,
                    ..
                }) = node_tree.get_mut(node_id).map(|node| node.get_mut())
                {
                    for a in attributes_to_add.drain(..) {
                        if let Some(dom) = dom {
                            dom.set_attribute(&a.0, &a.1).expect("set attribute error");
                        }
                        attributes.insert(a.0, a.1);
                    }
                }
            }
            Patch::DropAttributes(node_id, mut attributes_to_remove) => {
                if let Some(DomNode::Element {
                    dom,
                    ref mut attributes,
                    ..
                }) = node_tree.get_mut(node_id).map(|node| node.get_mut())
                {
                    for a in attributes_to_remove.drain(..) {
                        if let Some(dom) = dom {
                            dom.remove_attribute(&a).expect("remove attribute error");
                        }
                        attributes.remove(&a);
                    }
                }
            }
            Patch::SetEvents(node_id, mut events_to_add) => {
                if let Some(DomNode::Element {
                    dom,
                    ref mut events,
                    ..
                }) = node_tree.get_mut(node_id).map(|node| node.get_mut())
                {
                    for (t, f) in events_to_add.0.drain(..) {
                        debug!("ADD EVENT LISTENER");
                        if let Some(dom) = dom {
                            dom.add_event_listener_with_callback(
                                t.as_ref(),
                                f.as_ref().as_ref().unchecked_ref(),
                            )
                            .expect("add event error");
                        }
                        events.0.insert(t, f);
                    }
                }
            }
            Patch::UpdateEvents(node_id, mut events_to_update) => {
                if let Some(DomNode::Element {
                    dom,
                    ref mut events,
                    ..
                }) = node_tree.get_mut(node_id).map(|node| node.get_mut())
                {
                    for (t, f) in events_to_update.0.drain(..) {
                        if let Some(old_event) = events.0.get(&t) {
                            if let Some(dom) = dom {
                                dom.remove_event_listener_with_callback(
                                    t.as_ref(),
                                    old_event.as_ref().as_ref().unchecked_ref(),
                                )
                                .expect("remove event error");
                            }
                        }
                        if let Some(dom) = dom {
                            dom.add_event_listener_with_callback(
                                t.as_ref(),
                                f.as_ref().as_ref().unchecked_ref(),
                            )
                            .expect("add event error");
                        }
                        events.0.insert(t, f);
                    }
                }
            }
            Patch::DropEvents(node_id, mut events_to_remove) => {
                if let Some(DomNode::Element {
                    dom,
                    ref mut events,
                    ..
                }) = node_tree.get_mut(node_id).map(|node| node.get_mut())
                {
                    for (t, f) in events_to_remove.0.drain(..) {
                        if let Some(dom) = dom {
                            dom.remove_event_listener_with_callback(
                                t.as_ref(),
                                f.as_ref().as_ref().unchecked_ref(),
                            )
                            .expect("remove event error");
                        }
                        events.0.remove(&t);
                    }
                }
            }
            Patch::Append(node_id) => {
                if let Some((
                    node,
                    DomNode::Element {
                        dom: parent_dom, ..
                    },
                )) = node_tree
                    .get(node_id)
                    .and_then(|node| find_parent(node, node_tree).map(|parent| (node, parent)))
                {
                    if let Some(parent_dom) = parent_dom {
                        match node.get() {
                            DomNode::Element { dom, .. } => {
                                if let Some(dom) = dom {
                                    parent_dom
                                        .append_child(dom)
                                        .expect("append child element error");
                                }
                            }
                            DomNode::Text { dom, .. } => {
                                if let Some(dom) = dom {
                                    parent_dom
                                        .append_child(dom)
                                        .expect("append child text error");
                                }
                            }
                            DomNode::Component { .. } => {}
                        };
                    }
                }
            }
            Patch::AppendBefore(node_id, next_id) => {
                if let Some((
                    node,
                    next,
                    DomNode::Element {
                        dom: Some(parent_dom),
                        ..
                    },
                )) = node_tree
                    .get(node_id)
                    .and_then(|node| node_tree.get(next_id).map(|next| (node, next.get())))
                    .and_then(|(node, next)| {
                        find_parent(node, node_tree).map(|parent| (node.get(), next, parent))
                    })
                {
                    match (node, next) {
                        (
                            DomNode::Element { dom, .. },
                            DomNode::Element {
                                dom: Some(next_dom),
                                ..
                            },
                        ) => {
                            if let Some(dom) = dom {
                                parent_dom
                                    .insert_before(dom, Some(next_dom))
                                    .expect("insert before error");
                            }
                        }
                        (
                            DomNode::Text { dom, .. },
                            DomNode::Element {
                                dom: Some(next_dom),
                                ..
                            },
                        ) => {
                            if let Some(dom) = dom {
                                parent_dom
                                    .insert_before(dom, Some(next_dom))
                                    .expect("insert before error");
                            }
                        }
                        (
                            DomNode::Element { dom, .. },
                            DomNode::Text {
                                dom: Some(next_dom),
                                ..
                            },
                        ) => {
                            if let Some(dom) = dom {
                                parent_dom
                                    .insert_before(dom, Some(next_dom))
                                    .expect("insert before error");
                            }
                        }
                        (
                            DomNode::Text { dom, .. },
                            DomNode::Text {
                                dom: Some(next_dom),
                                ..
                            },
                        ) => {
                            if let Some(dom) = dom {
                                parent_dom
                                    .insert_before(dom, Some(next_dom))
                                    .expect("insert before error");
                            }
                        }
                        _ => {}
                    };
                }
            }
            Patch::Drop(node_id) => {
                if let Some((
                    node,
                    DomNode::Element {
                        dom: Some(parent_dom),
                        ..
                    },
                )) = node_tree.get(node_id).and_then(|node| {
                    find_parent(node, node_tree).map(|parent| (node.get(), parent))
                }) {
                    match node {
                        DomNode::Element { dom: Some(dom), .. } => {
                            parent_dom
                                .remove_child(dom)
                                .expect("remove child text error");
                        }
                        DomNode::Text { dom: Some(dom), .. } => {
                            parent_dom
                                .remove_child(dom)
                                .expect("remove child text error");
                        }
                        _ => {}
                    };
                };

                node_id.remove_subtree(node_tree);
            }
            Patch::DropFromToEnd(node_id) => {
                let mut subtree_to_remove: Vec<NodeId> = Vec::default();
                if let Some(DomNode::Element {
                    dom: parent_dom, ..
                }) = node_tree
                    .get(node_id)
                    .and_then(|node| find_parent(node, node_tree))
                {
                    for n in node_id.following_siblings(node_tree) {
                        if let Some(parent_dom) = parent_dom {
                            if let Some(node) = node_tree.get(n).map(|node| node.get()) {
                                match node {
                                    DomNode::Element { dom: Some(dom), .. } => {
                                        parent_dom.remove_child(dom).unwrap();
                                    }
                                    DomNode::Text { dom: Some(dom), .. } => {
                                        parent_dom.remove_child(dom).unwrap();
                                    }
                                    _ => {}
                                };
                            }
                        }
                        subtree_to_remove.push(n);
                    }
                }
                for n in subtree_to_remove {
                    n.remove_subtree(node_tree);
                }
            }
            Patch::DropChildren(node_id) => {
                if let Some(DomNode::Element { dom: Some(dom), .. }) =
                    node_tree.get(node_id).map(|node| node.get())
                {
                    for n in node_id.children(node_tree) {
                        if let Some(node) = node_tree.get(n).map(|node| node.get()) {
                            match node {
                                DomNode::Element {
                                    dom: Some(child_dom),
                                    ..
                                } => {
                                    dom.remove_child(child_dom).expect("remove child error");
                                }
                                DomNode::Text {
                                    dom: Some(child_dom),
                                    ..
                                } => {
                                    dom.remove_child(child_dom).expect("remove child error");
                                }
                                _ => {}
                            };
                        }
                    }
                };

                node_id.remove_subtree(node_tree);
            }
        }
    }
}

fn drop_remaining(
    mut current_node_id: NodeId,
    parent_limit: NodeId,
    node_tree: &Arena<DomNode>,
    patches: &mut Vec<Patch>,
) {
    while current_node_id != parent_limit {
        if let Some(next) = node_tree
            .get(current_node_id)
            .and_then(|current_node| current_node.next_sibling())
        {
            patches.push(Patch::DropFromToEnd(next));
            current_node_id = next;
        }
        current_node_id = node_tree
            .get(current_node_id)
            .and_then(|current_node| current_node.parent())
            .expect("parent not found");
    }
}

fn create_node_from_vnode(
    vnode: VNode,
    node_tree: &mut Arena<DomNode>,
    document: Option<&web_sys::Document>,
) -> (NodeId, Option<Vec<VNode>>) {
    match vnode {
        VNode::Element {
            tag,
            attributes,
            events,
            children,
        } => (
            node_tree.new_node(DomNode::Element {
                dom: create_element_dom(&tag, &attributes, &events, document),
                tag,
                attributes,
                events,
            }),
            Some(children),
        ),
        VNode::Text(text) => (
            node_tree.new_node(DomNode::Text {
                dom: create_text_dom(&text, document),
                text: text,
            }),
            None,
        ),
        VNode::Component(function) => {
            let mut hook_context = HookContext {
                hooks: Vec::default(),
                counter: 0,
            };
            let child = HOOK_CONTEXT.set(&mut hook_context, || vec![function.run()]);
            let new_node_id = node_tree.new_node(DomNode::Component {
                hooks: hook_context,
                function,
            });

            (new_node_id, Some(child))
        }
    }
}

fn reconcile(
    parent: NodeId,
    actual_children: Option<NodeId>,
    new_children: Option<VNode>,
    node_tree: &mut Arena<DomNode>,
    patches: &mut Vec<Patch>,
    document: Option<&web_sys::Document>,
) -> (NodeId, Option<Vec<VNode>>) {
    match (actual_children, new_children) {
        (None, Some(vnode)) => {
            let (new_node_id, children) = create_node_from_vnode(vnode, node_tree, document);

            parent.append(new_node_id, node_tree);
            patches.push(Patch::Append(new_node_id));

            debug!("add element fiber node id: {:?}", new_node_id);

            (new_node_id, children)
        }
        (Some(old_node), None) => {
            patches.push(Patch::Drop(old_node));

            debug!("drop node id: {:?}", old_node);

            (old_node, None)
        }
        (Some(old_node_id), Some(new_node)) => {
            match (node_tree.get_mut(old_node_id).unwrap().get_mut(), new_node) {
                (node, vnode) if node != &vnode => {
                    let (new_node_id, children) =
                        create_node_from_vnode(vnode, node_tree, document);

                    old_node_id.insert_after(new_node_id, node_tree);
                    patches.push(Patch::AppendBefore(new_node_id, old_node_id));
                    patches.push(Patch::Drop(old_node_id));

                    debug!("change element tree node id: {:?}", new_node_id);

                    (new_node_id, children)
                }
                (
                    DomNode::Element {
                        tag: old_tag,
                        attributes: old_attributes,
                        events: old_events,
                        ..
                    },
                    VNode::Element {
                        tag: new_tag,
                        attributes: mut new_attributes,
                        events: mut new_events,
                        children,
                    },
                ) if old_tag == &new_tag && old_tag != &Tag::Empty => {
                    let mut attributes_to_set: Vec<(String, String)> = Vec::new();
                    let attributes_to_remove: Vec<String> = old_attributes
                        .keys()
                        .filter_map(|k| {
                            if !new_attributes.contains_key(k) {
                                Some(k.to_owned())
                            } else {
                                None
                            }
                        })
                        .collect();
                    for (k, v) in new_attributes.drain() {
                        match old_attributes.get(&k) {
                            Some(value) if value != &v => {
                                attributes_to_set.push((k, v));
                            }
                            None => {
                                attributes_to_set.push((k, v));
                            }
                            _ => {}
                        }
                    }

                    if attributes_to_set.len() > 0 {
                        patches.push(Patch::SetAttributes(old_node_id, attributes_to_set));
                    }
                    if attributes_to_remove.len() > 0 {
                        patches.push(Patch::DropAttributes(old_node_id, attributes_to_remove));
                    }

                    let mut events_to_set: Vec<(EventType, DynClosure)> = Vec::new();
                    let mut events_to_update: Vec<(EventType, DynClosure)> = Vec::new();
                    let events_to_remove: Vec<(EventType, DynClosure)> = old_events
                        .0
                        .iter()
                        .filter_map(|(k, v)| {
                            if !new_events.0.contains_key(k) {
                                Some((k.clone(), v.clone()))
                            } else {
                                None
                            }
                        })
                        .collect();
                    for (k, v) in new_events.0.drain() {
                        match old_events.0.get(&k) {
                            Some(value) if !Rc::ptr_eq(value, &v) => {
                                events_to_update.push((k, v));
                            }
                            None => {
                                events_to_set.push((k, v));
                            }
                            _ => {}
                        }
                    }

                    if events_to_set.len() > 0 {
                        patches.push(Patch::SetEvents(old_node_id, EventsVec(events_to_set)));
                    }
                    if events_to_update.len() > 0 {
                        patches.push(Patch::UpdateEvents(
                            old_node_id,
                            EventsVec(events_to_update),
                        ));
                    }
                    if events_to_remove.len() > 0 {
                        patches.push(Patch::DropEvents(old_node_id, EventsVec(events_to_remove)));
                    }

                    if children.is_empty() && old_node_id.children(node_tree).next().is_some() {
                        patches.push(Patch::DropChildren(old_node_id));
                    }

                    (old_node_id, Some(children))
                }
                (DomNode::Component { hooks, function }, VNode::Component(component))
                    if function == &component =>
                {
                    hooks.counter = 0;
                    HOOK_CONTEXT.set(hooks, || {
                        let child = function.run();
                        (old_node_id, Some(vec![child]))
                    })
                }
                (_, VNode::Element { children, .. }) => {
                    if children.is_empty() {
                        patches.push(Patch::DropChildren(old_node_id));
                    }

                    (old_node_id, Some(children))
                }
                _ => (old_node_id, None),
            }
        }
        _ => panic!("should never happen"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::component;
    use crate::component::create_component;
    use crate::component::ComponentProvider;
    use crate::tag::Tag;
    use crate::VNode;
    use crate::{create_element, create_text};
    use indextree::{Arena, NodeEdge};

    fn create_app(dom: VNode) -> App {
        let mut tree = Arena::default();
        let root_node = tree.new_node(DomNode::Element {
            dom: None,
            tag: Tag::Empty,
            attributes: HashMap::with_capacity(0),
            events: Events(HashMap::with_capacity(0)),
        });
        App {
            node_tree: tree,
            root: root_node,
            working_context: WorkingContext {
                vnode_buffer: vec![(
                    0,
                    VNode::Element {
                        tag: Tag::Empty,
                        attributes: HashMap::with_capacity(0),
                        events: Events(HashMap::with_capacity(0)),
                        children: vec![dom],
                    },
                )],
                patches: Vec::default(),
                last_deep: 0,
                current_node_id: root_node,
                last_parent: root_node,
            },
            document: None,
        }
    }

    fn work_on_dom(app: &mut App) {
        work_on_vdom(app, || true);
    }

    fn manually_generate_working_context(app: &mut App, vdom: VNode) {
        app.working_context = WorkingContext {
            vnode_buffer: vec![(
                0,
                VNode::Element {
                    tag: Tag::Empty,
                    attributes: HashMap::with_capacity(0),
                    events: Events(HashMap::with_capacity(0)),
                    children: vec![vdom],
                },
            )],
            patches: Vec::default(),
            last_deep: 0,
            current_node_id: app.root,
            last_parent: app.root,
        };
    }

    fn compare_vdom_with_dom(vdom: VNode, app: &App) {
        println!("vdom: {:?}", vdom);
        let mut vnode_buffer = vec![VNode::Element {
            tag: Tag::Empty,
            attributes: HashMap::with_capacity(0),
            events: Events(HashMap::with_capacity(0)),
            children: vec![vdom],
        }];
        let mut node_buffer = vec![app.root];
        while !(node_buffer.is_empty()) {
            if let Some(node_id) = node_buffer.pop() {
                for c in node_id.children(&app.node_tree) {
                    if !c.is_removed(&app.node_tree) {
                        node_buffer.push(c);
                    }
                }

                if let Some(vnode) = vnode_buffer.pop() {
                    match (vnode, app.node_tree.get(node_id).map(|n| n.get())) {
                        (
                            VNode::Element {
                                tag: vtag,
                                attributes: vattributes,
                                events: vevents,
                                children,
                            },
                            Some(DomNode::Element {
                                tag,
                                attributes,
                                events,
                                ..
                            }),
                        ) => {
                            for c in children.into_iter() {
                                vnode_buffer.push(c);
                            }
                            println!("vnode: {:?}, {:?}, {:?}", vtag, vattributes, vevents);
                            println!("node: {:?}, {:?}, {:?}", tag, attributes, events);
                            assert_eq!(vtag, *tag);
                            assert_eq!(vattributes, *attributes);
                            assert_eq!(vevents, *events);
                        }
                        (VNode::Text(vtext), Some(DomNode::Text { text, .. })) => {
                            println!("vnode: {:?}", vtext);
                            println!("node: {:?}", text);
                            assert_eq!(vtext, *text);
                        }
                        (VNode::Component(component), Some(DomNode::Component { .. })) => {
                            vnode_buffer.push(component.run());
                        }
                        _ => {
                            panic!("Different node");
                        }
                    }
                } else {
                    panic!("No vnode found");
                }
            }
        }
    }

    fn print_tree(app: &App) {
        println!("print tree: ");
        let mut deep = 0;
        for i in app.root.traverse(&app.node_tree) {
            match i {
                NodeEdge::Start(node) => {
                    if !node.is_removed(&app.node_tree) {
                        for _ in 0..deep {
                            print!("-");
                        }
                        println!(
                            "nodeId: {:?}, value: {:?}",
                            node,
                            app.node_tree.get(node).map(|node| node.get())
                        );
                        deep += 1;
                    }
                }
                NodeEdge::End(_) => deep -= 1,
            }
        }
        println!();
    }

    #[test]
    fn simple_vdom_creation() {
        let vdom = || {
            create_element(Tag::Div)
                .with_child(create_text("hello world"))
                .build()
        };
        let mut app = create_app(vdom());

        work_on_dom(&mut app);
        commit_patches(&mut app);

        compare_vdom_with_dom(vdom(), &app);
    }

    #[test]
    fn complex_vdom_creation() {
        let vdom = || {
            create_element(Tag::Div)
                .with_attribute("id", "foo")
                .with_child(
                    create_element(Tag::A)
                        .with_child(
                            create_element(Tag::Div)
                                .with_child(
                                    create_element(Tag::Div)
                                        .with_child(create_text("bar"))
                                        .build(),
                                )
                                .build(),
                        )
                        .build(),
                )
                .with_child(create_element(Tag::B).build())
                .build()
        };
        let mut app = create_app(vdom());

        work_on_dom(&mut app);
        commit_patches(&mut app);

        print_tree(&app);

        compare_vdom_with_dom(vdom(), &app);
    }

    #[test]
    fn no_changes() {
        let vdom = || {
            create_element(Tag::Div)
                .with_attribute("id", "foo")
                .with_child(
                    create_element(Tag::A)
                        .with_child(create_element(Tag::Div).build())
                        .build(),
                )
                .build()
        };
        let mut app = create_app(vdom());

        work_on_dom(&mut app);
        assert_eq!(app.working_context.patches.len(), 3);
        commit_patches(&mut app);

        app.start_new_work();
        work_on_dom(&mut app);
        assert_eq!(app.working_context.patches.len(), 0);
        commit_patches(&mut app);
    }

    #[test]
    fn change_text() {
        let vdom = || create_text("hello world");
        let mut app = create_app(vdom());

        work_on_dom(&mut app);

        commit_patches(&mut app);

        compare_vdom_with_dom(vdom(), &app);

        let vdom = || create_text("hello world 2");

        manually_generate_working_context(&mut app, vdom());

        work_on_dom(&mut app);
        commit_patches(&mut app);

        compare_vdom_with_dom(vdom(), &app);
    }

    #[test]
    fn remove_last_element() {
        let vdom = || {
            create_element(Tag::Div)
                .with_attribute("id", "foo")
                .with_child(
                    create_element(Tag::A)
                        .with_child(create_element(Tag::Div).build())
                        .build(),
                )
                .build()
        };
        let mut app = create_app(vdom());

        work_on_dom(&mut app);
        commit_patches(&mut app);

        compare_vdom_with_dom(vdom(), &app);

        let vdom = || {
            create_element(Tag::Div)
                .with_attribute("id", "foo")
                .with_child(create_element(Tag::A).build())
                .build()
        };
        manually_generate_working_context(&mut app, vdom());
        work_on_dom(&mut app);
        commit_patches(&mut app);

        compare_vdom_with_dom(vdom(), &app);
    }

    #[test]
    fn insert_in_the_middle() {
        let vdom = || {
            create_element(Tag::Div)
                .with_child(create_element(Tag::Div).build())
                .with_child(create_element(Tag::Div).build())
                .with_child(create_element(Tag::Div).build())
                .with_child(create_element(Tag::Div).build())
                .build()
        };
        let mut app = create_app(vdom());

        work_on_dom(&mut app);
        commit_patches(&mut app);

        print_tree(&app);

        compare_vdom_with_dom(vdom(), &app);

        let vdom = || {
            create_element(Tag::Div)
                .with_child(create_element(Tag::Div).build())
                .with_child(create_element(Tag::Button).build())
                .with_child(create_element(Tag::Div).build())
                .with_child(create_element(Tag::Div).build())
                .with_child(create_element(Tag::Div).build())
                .build()
        };
        manually_generate_working_context(&mut app, vdom());

        work_on_dom(&mut app);
        commit_patches(&mut app);

        print_tree(&app);
        println!("{:?}", vdom());

        compare_vdom_with_dom(vdom(), &app);
    }

    #[test]
    fn drop_remaining() {
        let vdom = || {
            create_element(Tag::Div)
                .with_child(create_element(Tag::Div).build())
                .with_child(create_element(Tag::Div).build())
                .with_child(create_element(Tag::Div).build())
                .with_child(create_element(Tag::Div).build())
                .build()
        };
        let mut app = create_app(vdom());

        work_on_dom(&mut app);
        commit_patches(&mut app);

        print_tree(&app);

        compare_vdom_with_dom(vdom(), &app);

        let vdom = || {
            create_element(Tag::Div)
                .with_child(create_element(Tag::Div).build())
                .with_child(create_element(Tag::Div).build())
                .build()
        };
        manually_generate_working_context(&mut app, vdom());

        work_on_dom(&mut app);
        commit_patches(&mut app);

        print_tree(&app);

        compare_vdom_with_dom(vdom(), &app);
    }

    #[derive(PartialEq, Debug)]
    pub struct TestProp {
        index: usize,
    }

    #[component(TestComponent)]
    fn test_component(props: &TestProp) -> VNode {
        create_text(&format!("test {}", props.index))
    }

    #[component(TestComponent2)]
    fn test_component2(props: &TestProp) -> VNode {
        create_element(Tag::Div)
            .with_child(create_text(&format!("test {}", props.index)))
            .build()
    }

    #[test]
    fn component_test() {
        let vdom = || {
            create_element(Tag::Div)
                .with_child(create_component::<TestComponent>(TestProp { index: 5 }))
                .build()
        };
        let mut app = create_app(vdom());

        work_on_dom(&mut app);
        commit_patches(&mut app);

        print_tree(&app);

        compare_vdom_with_dom(vdom(), &app);

        let vdom = || {
            create_element(Tag::Div)
                .with_child(create_element(Tag::Div).build())
                .with_child(create_element(Tag::Div).build())
                .build()
        };
        manually_generate_working_context(&mut app, vdom());

        work_on_dom(&mut app);
        commit_patches(&mut app);

        print_tree(&app);

        compare_vdom_with_dom(vdom(), &app);
    }

    #[test]
    fn component_test_changes() {
        let vdom = || {
            create_element(Tag::Div)
                .with_child(create_component::<TestComponent>(TestProp { index: 5 }))
                .build()
        };
        let mut app = create_app(vdom());

        work_on_dom(&mut app);
        commit_patches(&mut app);

        print_tree(&app);

        compare_vdom_with_dom(vdom(), &app);

        let vdom = || {
            create_element(Tag::Div)
                .with_child(create_component::<TestComponent2>(TestProp { index: 3 }))
                .build()
        };
        manually_generate_working_context(&mut app, vdom());

        work_on_dom(&mut app);
        commit_patches(&mut app);

        print_tree(&app);

        compare_vdom_with_dom(vdom(), &app);
    }
}
