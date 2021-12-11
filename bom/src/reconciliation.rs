use crate::fiber::{EffectTag, FiberId, FiberNode, FiberTree, UpdateData};
use crate::node::{Component, Element, Node, Text};
use crate::VElement;
use crate::VNode;
use crate::HOOK_CONTEXT;

pub(crate) fn perform_unit_of_work(
    id: FiberId,
    fiber_tree: &mut FiberTree,
    document: Option<&web_sys::Document>,
) -> Option<FiberId> {
    if fiber_tree
        .get(id)
        .map(|fiber_node| {
            if let Node::Component(Component { .. }) = fiber_node.node {
                true
            } else {
                false
            }
        })
        .unwrap_or(false)
    {
        update_component_node(id, fiber_tree)
    } else {
        update_node(id, fiber_tree, document);
    }

    if let Some(child) = fiber_tree.get(id).and_then(|fiber_node| fiber_node.child) {
        Some(child)
    } else {
        let mut next_fiber_id = Some(id);
        while let Some(some_next_fiber_id) = next_fiber_id {
            let fiber_node = fiber_tree.get(some_next_fiber_id);
            if let Some(sibling) = fiber_node.and_then(|fiber_node| fiber_node.sibling) {
                return Some(sibling);
            }

            next_fiber_id = fiber_node.and_then(|fiber_node| fiber_node.parent)
        }

        None
    }
}

fn update_component_node(id: FiberId, fiber_tree: &mut FiberTree) {
    if let Some(Node::Component(Component { hooks, function })) = fiber_tree
        .get_mut(id)
        .map(|fiber_node| &mut fiber_node.node)
    {
        hooks.counter = 0;
        let hooks = (id, hooks);
        let elements = HOOK_CONTEXT.set(hooks, || vec![function.run()]);
        reconcile_children(id, elements, fiber_tree);
    }
}

fn update_node(id: FiberId, fiber_tree: &mut FiberTree, document: Option<&web_sys::Document>) {
    let element = fiber_tree.get_mut(id).unwrap();
    if match &element.node {
        Node::Element(Element { dom, .. }) => dom.is_none(),
        Node::Text(Text { dom, .. }) => dom.is_none(),
        _ => false,
    } {
        element.node.create_dom(document);
    }
    if let Node::Element(Element {
        unprocessed_children,
        ..
    }) = &mut element.node
    {
        let mut elements: Vec<VNode> = Vec::default();
        for e in unprocessed_children.drain(..) {
            elements.push(e);
        }

        reconcile_children(id, elements, fiber_tree)
    }
}

fn reconcile_children(id: FiberId, elements: Vec<VNode>, fiber_tree: &mut FiberTree) {
    let wip_fiber_id = id;
    let mut current_id: Option<FiberId> = fiber_tree.get(wip_fiber_id).and_then(|node| node.child);
    let mut prev_sibling: Option<FiberId> = None;

    for element in elements {
        let new_node = {
            let current_fiber = current_id.and_then(|id: FiberId| fiber_tree.get_mut(id));

            match current_fiber {
                Some(old_fiber) if old_fiber == &element => {
                    match element {
                        VNode::Element(VElement {
                            attributes,
                            events,
                            children,
                            ..
                        }) => {
                            old_fiber.effect_tag = Some(EffectTag::Update(UpdateData::Element {
                                attributes,
                                events,
                            }));
                            if let Node::Element(Element {
                                unprocessed_children,
                                ..
                            }) = &mut old_fiber.node
                            {
                                *unprocessed_children = children;
                            }
                        }
                        VNode::Text(text) => {
                            old_fiber.effect_tag = Some(EffectTag::Update(UpdateData::Text(text)));
                        }
                        _ => {}
                    };
                    old_fiber.parent = Some(wip_fiber_id);

                    None
                }
                Some(old_fiber) if old_fiber != &element => Some(element.to_node()),
                None => Some(element.to_node()),
                _ => None,
            }
        };

        if let Some(new_node) = new_node {
            let new_node_id = fiber_tree.new_node(new_node);

            fiber_tree.insert_child(new_node_id, wip_fiber_id, prev_sibling);
            prev_sibling = Some(new_node_id);
        } else {
            prev_sibling = current_id;
        }

        current_id = prev_sibling
            .and_then(|prev_sibling| fiber_tree.get(prev_sibling))
            .and_then(|fiber| fiber.sibling);
    }

    while let Some(current) = current_id {
        if let Some(old_fiber) = fiber_tree.get_mut(current) {
            old_fiber.effect_tag = Some(EffectTag::Deletion);
        }

        current_id = fiber_tree.get(current).and_then(|node| node.sibling);
    }
}

fn find_first_element_parent(node_id: FiberId, fiber_tree: &FiberTree) -> Option<FiberId> {
    let mut parent_id = fiber_tree
        .get(node_id)
        .and_then(|fiber_node| fiber_node.parent);

    while let Some(some_parent_id) = parent_id {
        if let Some(parent_node) = fiber_tree.get(some_parent_id) {
            match &parent_node.node {
                Node::Element(_) => {
                    return Some(some_parent_id);
                }
                _ => {
                    parent_id = parent_node.parent;
                }
            }
        } else {
            return None;
        }
    }

    None
}

fn find_first_dom_child(node_id: FiberId, fiber_tree: &FiberTree) -> Option<FiberId> {
    let mut child_id = fiber_tree
        .get(node_id)
        .and_then(|fiber_node| fiber_node.child);

    while let Some(some_child_id) = child_id {
        if let Some(child_node) = fiber_tree.get(some_child_id) {
            match &child_node.node {
                Node::Element(_) => {
                    return Some(some_child_id);
                }
                Node::Text(_) => {
                    return Some(some_child_id);
                }
                _ => {
                    child_id = child_node.child;
                }
            }
        } else {
            return None;
        }
    }

    None
}

pub(crate) fn commit(id: Option<FiberId>, fiber_tree: &mut FiberTree) {
    if let Some(id) = id {
        if let Some(parent_id) = find_first_element_parent(id, fiber_tree) {
            let parent_dom =
                fiber_tree
                    .get(parent_id)
                    .and_then(|parent_node| match &parent_node.node {
                        Node::Element(Element { dom, .. }) => dom.clone(),
                        _ => None,
                    });

            if let Some(effect_tag) = fiber_tree
                .get_mut(id)
                .and_then(|fiber_node| fiber_node.effect_tag.take())
            {
                match effect_tag {
                    EffectTag::Placement => match (
                        fiber_tree.get(id).map(|fiber_node| &fiber_node.node),
                        parent_dom,
                    ) {
                        (Some(Node::Element(Element { dom: Some(dom), .. })), Some(parent_dom)) => {
                            parent_dom
                                .append_child(&dom)
                                .expect("append element child error");
                        }
                        (Some(Node::Text(Text { dom: Some(dom), .. })), Some(parent_dom)) => {
                            parent_dom
                                .append_child(&dom)
                                .expect("append text child error");
                        }
                        _ => {}
                    },
                    EffectTag::Update(UpdateData::Element { attributes, events }) => {
                        match fiber_tree
                            .get_mut(id)
                            .map(|fiber_node| &mut fiber_node.node)
                        {
                            Some(Node::Element(element)) => {
                                element.update_element_dom(attributes, events)
                            }
                            _ => {}
                        }
                    }
                    EffectTag::Update(UpdateData::Text(new_text)) => match fiber_tree
                        .get_mut(id)
                        .map(|fiber_node| &mut fiber_node.node)
                    {
                        Some(Node::Text(text)) => text.update_text_dom(new_text),
                        _ => {}
                    },
                    EffectTag::Deletion => {
                        match (fiber_tree.get(id), parent_dom) {
                            (
                                Some(FiberNode {
                                    node: Node::Element(Element { dom: Some(dom), .. }),
                                    ..
                                }),
                                Some(parent_dom),
                            ) => {
                                parent_dom
                                    .remove_child(dom)
                                    .expect("remove element child error");
                            }
                            (
                                Some(FiberNode {
                                    node: Node::Text(Text { dom: Some(dom), .. }),
                                    ..
                                }),
                                Some(parent_dom),
                            ) => {
                                parent_dom
                                    .remove_child(dom)
                                    .expect("remove text child error");
                            }
                            (
                                Some(FiberNode {
                                    node: Node::Component(_),
                                    ..
                                }),
                                Some(parent_dom),
                            ) => match find_first_dom_child(id, fiber_tree)
                                .and_then(|child_id| fiber_tree.get(child_id))
                                .map(|child_node| &child_node.node)
                            {
                                Some(Node::Element(Element { dom: Some(dom), .. })) => {
                                    parent_dom
                                        .remove_child(dom)
                                        .expect("remove first component element child error");
                                }
                                Some(Node::Text(Text { dom: Some(dom), .. })) => {
                                    parent_dom
                                        .remove_child(dom)
                                        .expect("remove first component text child error");
                                }
                                _ => {}
                            },

                            _ => {}
                        }
                        fiber_tree.remove(id);
                    }
                }
            }
        }
        commit(fiber_tree.get(id).and_then(|node| node.child), fiber_tree);
        commit(fiber_tree.get(id).and_then(|node| node.sibling), fiber_tree);
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::app::App;
//     use crate::commit_work;
//     use crate::component;
//     use crate::component::create_component;
//     use crate::component::ComponentProvider;
//     use crate::tag::Tag;
//     use crate::work_loop;
//     use crate::Events;
//     use crate::VNode;
//     //use crate::{create_element, create_text};
//     use std::collections::HashMap;

//     fn create_app(dom: VNode) -> App {
//         let mut fiber_tree = FiberTree::default();
//         let root_id = fiber_tree.new_node(Node::Element(Element {
//             dom: None,
//             tag: Tag::Empty,
//             attributes: HashMap::with_capacity(0),
//             events: Events(HashMap::with_capacity(0)),
//             unprocessed_children: vec![dom],
//         }));
//         App {
//             fiber_tree,
//             root: root_id,
//             next_unit_of_work: Some(root_id),
//             wip_root: Some(root_id),
//             document: None,
//         }
//     }

//     fn work_on_dom(app: &mut App) {
//         work_loop(app, || true);
//     }

//     fn commit(app: &mut App) {
//         commit_work(app, || true);
//     }

//     fn compare_vdom_with_dom(vdom: VNode, app: &App) {
//         println!("vdom: {:?}", vdom);
//         let mut vnode_buffer = vec![VNode::Element {
//             tag: Tag::Empty,
//             attributes: HashMap::with_capacity(0),
//             events: Events(HashMap::with_capacity(0)),
//             children: vec![vdom],
//         }];
//         let mut node_buffer = vec![app.root];
//         while !(node_buffer.is_empty()) {
//             if let Some(node_id) = node_buffer.pop() {
//                 let mut next = app.fiber_tree.get(node_id).and_then(|node| node.child);
//                 while let Some(current) = next {
//                     node_buffer.push(current);
//                     next = app.fiber_tree.get(current).and_then(|node| node.sibling);
//                 }
//                 if let Some(vnode) = vnode_buffer.pop() {
//                     match (vnode, app.fiber_tree.get(node_id).map(|n| &n.node)) {
//                         (
//                             VNode::Element {
//                                 tag: vtag,
//                                 attributes: vattributes,
//                                 events: vevents,
//                                 children,
//                             },
//                             Some(Node::Element(Element {
//                                 tag,
//                                 attributes,
//                                 events,
//                                 ..
//                             })),
//                         ) => {
//                             for c in children.into_iter() {
//                                 vnode_buffer.push(c);
//                             }
//                             println!("vnode: {:?}, {:?}, {:?}", vtag, vattributes, vevents);
//                             println!("node: {:?}, {:?}, {:?}", tag, attributes, events);
//                             assert_eq!(vtag, *tag);
//                             assert_eq!(vattributes, *attributes);
//                             assert_eq!(vevents, *events);
//                         }
//                         (VNode::Text(vtext), Some(Node::Text(Text { text, .. }))) => {
//                             println!("vnode: {:?}", vtext);
//                             println!("node: {:?}", text);
//                             assert_eq!(vtext, *text);
//                         }
//                         (VNode::Component(component), Some(Node::Component(Component { .. }))) => {
//                             vnode_buffer.push(component.run());
//                         }
//                         _ => {
//                             panic!("Different node");
//                         }
//                     }
//                 } else {
//                     panic!("No vnode found");
//                 }
//             }
//         }
//     }

//     fn print_tree(app: &App) {
//         println!("print tree: ");
//         let mut next = Some(app.root);
//         let mut deep = 0;
//         while let Some(current) = next {
//             for _ in 0..deep {
//                 print!("-");
//             }
//             println!(
//                 "nodeId: {:?}, value: {:?}",
//                 current,
//                 app.fiber_tree.get(current).map(|node| &node.node)
//             );

//             next = app.fiber_tree.get(current).and_then(|node| node.child);
//             if let Some(child) = app.fiber_tree.get(current).and_then(|node| node.child) {
//                 deep += 1;
//                 next = Some(child);
//             } else if let Some(sibling) = app.fiber_tree.get(current).and_then(|node| node.sibling)
//             {
//                 next = Some(sibling)
//             } else {
//                 deep -= 1;
//                 let mut go_up = app.fiber_tree.get(current).and_then(|node| node.parent);
//                 while let Some(current) = go_up {
//                     if let Some(sibling) = app.fiber_tree.get(current).and_then(|node| node.sibling)
//                     {
//                         next = Some(sibling);
//                         go_up = None;
//                     } else {
//                         deep -= 1;
//                         go_up = app.fiber_tree.get(current).and_then(|node| node.parent);
//                     }
//                 }
//             }
//         }
//         println!();
//     }

//     fn manually_generate_working_context(app: &mut App, vdom: VNode) {
//         app.wip_root = Some(app.root);
//         app.next_unit_of_work = Some(app.root);
//         app.fiber_tree
//             .get_mut(app.root)
//             .map(|node| match &mut node.node {
//                 Node::Element(Element {
//                     unprocessed_children,
//                     ..
//                 }) => {
//                     *unprocessed_children = vec![vdom];
//                 }
//                 _ => {}
//             });
//     }

//     #[test]
//     fn simple_vdom_creation() {
//         let vdom = || {
//             create_element(Tag::Div)
//                 .with_child(create_text("hello world"))
//                 .build()
//         };
//         let mut app = create_app(vdom());

//         work_on_dom(&mut app);
//         commit(&mut app);

//         compare_vdom_with_dom(vdom(), &app);
//     }

//     #[test]
//     fn complex_vdom_creation() {
//         let vdom = || {
//             create_element(Tag::Div)
//                 .with_attribute("id", "foo")
//                 .with_child(
//                     create_element(Tag::A)
//                         .with_child(
//                             create_element(Tag::Div)
//                                 .with_child(
//                                     create_element(Tag::Div)
//                                         .with_child(create_text("bar"))
//                                         .build(),
//                                 )
//                                 .build(),
//                         )
//                         .build(),
//                 )
//                 .with_child(create_element(Tag::B).build())
//                 .build()
//         };
//         let mut app = create_app(vdom());

//         work_on_dom(&mut app);
//         commit(&mut app);

//         print_tree(&app);

//         compare_vdom_with_dom(vdom(), &app);
//     }

//     #[test]
//     fn change_text() {
//         let vdom = || create_text("hello world");
//         let mut app = create_app(vdom());

//         work_on_dom(&mut app);
//         commit(&mut app);

//         compare_vdom_with_dom(vdom(), &app);

//         let vdom = || create_text("hello world 2");

//         manually_generate_working_context(&mut app, vdom());

//         work_on_dom(&mut app);
//         commit(&mut app);

//         compare_vdom_with_dom(vdom(), &app);
//     }

//     #[test]
//     fn remove_last_element() {
//         let vdom = || {
//             create_element(Tag::Div)
//                 .with_attribute("id", "foo")
//                 .with_child(
//                     create_element(Tag::A)
//                         .with_child(create_element(Tag::Div).build())
//                         .build(),
//                 )
//                 .build()
//         };
//         let mut app = create_app(vdom());

//         work_on_dom(&mut app);
//         commit(&mut app);

//         compare_vdom_with_dom(vdom(), &app);

//         let vdom = || {
//             create_element(Tag::Div)
//                 .with_attribute("id", "foo")
//                 .with_child(create_element(Tag::A).build())
//                 .build()
//         };
//         manually_generate_working_context(&mut app, vdom());

//         work_on_dom(&mut app);
//         commit(&mut app);

//         compare_vdom_with_dom(vdom(), &app);
//     }

//     #[test]
//     fn insert_in_the_middle() {
//         let vdom = || {
//             create_element(Tag::Div)
//                 .with_child(create_element(Tag::Div).build())
//                 .with_child(create_element(Tag::Div).build())
//                 .with_child(create_element(Tag::Div).build())
//                 .with_child(create_element(Tag::Div).build())
//                 .build()
//         };
//         let mut app = create_app(vdom());

//         work_on_dom(&mut app);
//         commit(&mut app);

//         print_tree(&app);

//         compare_vdom_with_dom(vdom(), &app);

//         let vdom = || {
//             create_element(Tag::Div)
//                 .with_child(create_element(Tag::Div).build())
//                 .with_child(create_element(Tag::Button).build())
//                 .with_child(create_element(Tag::Div).build())
//                 .with_child(create_element(Tag::Div).build())
//                 .with_child(create_element(Tag::Div).build())
//                 .build()
//         };
//         manually_generate_working_context(&mut app, vdom());

//         work_on_dom(&mut app);
//         commit(&mut app);

//         print_tree(&app);
//         println!("{:?}", vdom());

//         compare_vdom_with_dom(vdom(), &app);
//     }

//     #[test]
//     fn drop_remaining() {
//         let vdom = || {
//             create_element(Tag::Div)
//                 .with_child(create_element(Tag::Div).build())
//                 .with_child(create_element(Tag::Div).build())
//                 .with_child(create_element(Tag::Div).build())
//                 .with_child(create_element(Tag::Div).build())
//                 .build()
//         };
//         let mut app = create_app(vdom());

//         work_on_dom(&mut app);
//         commit(&mut app);

//         print_tree(&app);

//         compare_vdom_with_dom(vdom(), &app);

//         let vdom = || {
//             create_element(Tag::Div)
//                 .with_child(create_element(Tag::Div).build())
//                 .with_child(create_element(Tag::Div).build())
//                 .build()
//         };
//         manually_generate_working_context(&mut app, vdom());

//         work_on_dom(&mut app);
//         commit(&mut app);

//         print_tree(&app);

//         compare_vdom_with_dom(vdom(), &app);
//     }

//     #[derive(PartialEq, Debug)]
//     pub struct TestProp {
//         index: usize,
//     }

//     #[component(TestComponent)]
//     fn test_component(props: &TestProp) -> VNode {
//         create_text(&format!("test {}", props.index))
//     }

//     #[component(TestComponent2)]
//     fn test_component2(props: &TestProp) -> VNode {
//         create_element(Tag::Div)
//             .with_child(create_text(&format!("test {}", props.index)))
//             .build()
//     }

//     #[test]
//     fn component_test() {
//         let vdom = || {
//             create_element(Tag::Div)
//                 .with_child(create_component::<TestComponent>(TestProp { index: 5 }))
//                 .build()
//         };
//         let mut app = create_app(vdom());

//         work_on_dom(&mut app);
//         commit(&mut app);

//         print_tree(&app);

//         compare_vdom_with_dom(vdom(), &app);

//         let vdom = || {
//             create_element(Tag::Div)
//                 .with_child(create_element(Tag::Div).build())
//                 .with_child(create_element(Tag::Div).build())
//                 .build()
//         };
//         manually_generate_working_context(&mut app, vdom());

//         work_on_dom(&mut app);
//         commit(&mut app);

//         print_tree(&app);

//         compare_vdom_with_dom(vdom(), &app);
//     }

//     #[test]
//     fn component_test_changes() {
//         let vdom = || {
//             create_element(Tag::Div)
//                 .with_child(create_component::<TestComponent>(TestProp { index: 5 }))
//                 .build()
//         };
//         let mut app = create_app(vdom());

//         work_on_dom(&mut app);
//         commit(&mut app);

//         print_tree(&app);

//         compare_vdom_with_dom(vdom(), &app);

//         let vdom = || {
//             create_element(Tag::Div)
//                 .with_child(create_component::<TestComponent2>(TestProp { index: 3 }))
//                 .build()
//         };
//         manually_generate_working_context(&mut app, vdom());

//         work_on_dom(&mut app);
//         commit(&mut app);

//         print_tree(&app);

//         compare_vdom_with_dom(vdom(), &app);
//     }
// }
