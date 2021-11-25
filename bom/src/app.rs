use crate::dom::commit_patches;
use crate::dom::work_on_vdom;
use crate::dom::DomNode;
use crate::dom::WorkingContext;
use crate::request_idle_callback;
use crate::tag::Tag;
use crate::Events;
use crate::VNode;
use crate::HOOK_CONTEXT;
use indextree::Arena;
use indextree::NodeId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;

#[derive(Debug)]
pub(crate) struct App {
    pub node_tree: Arena<DomNode>,
    pub root: NodeId,
    pub working_context: WorkingContext,
    pub document: Option<web_sys::Document>,
}

impl App {
    pub fn start_new_work(&mut self) {
        self.working_context.vnode_buffer.clear();
        self.working_context.patches.clear();

        let node_id = self
            .node_tree
            .get(self.root)
            .unwrap()
            .first_child()
            .unwrap();

        self.working_context.last_deep = 0;
        self.working_context.current_node_id = node_id;
        self.working_context.last_parent = self.root;

        if let DomNode::Component { function, hooks } = self
            .node_tree
            .get_mut(
                self.node_tree
                    .get(self.root)
                    .unwrap()
                    .first_child()
                    .unwrap(),
            )
            .unwrap()
            .get_mut()
        {
            hooks.counter = 0;
            let child = HOOK_CONTEXT.set(hooks, || {
                let child = function.run();
                child
            });
            match child {
                VNode::Element {
                    tag,
                    attributes,
                    events,
                    children,
                } => {
                    self.working_context.vnode_buffer = vec![(
                        1,
                        VNode::Element {
                            tag: tag,
                            attributes: attributes,
                            events: events,
                            children: children,
                        },
                    )];
                }
                VNode::Text(text) => {
                    self.working_context.vnode_buffer = vec![(1, VNode::Text(text))];
                }
                VNode::Component(component) => {
                    self.working_context.vnode_buffer = vec![(1, VNode::Component(component))];
                }
            }
        }
    }
}

thread_local! {
    pub(crate) static APP: RefCell<Option<App>> = RefCell::new(None);
}

pub fn render(element: VNode, container: web_sys::Element) {
    APP.with(|app| {
        //let mut app = app.borrow_mut();
        let mut node_tree = Arena::default();
        let root_node = node_tree.new_node(DomNode::Element {
            dom: Some(container),
            tag: Tag::Empty,
            attributes: HashMap::with_capacity(0),
            events: Events(HashMap::with_capacity(0)),
        });
        app.replace(Some(App {
            node_tree: node_tree,
            root: root_node,
            working_context: WorkingContext {
                vnode_buffer: vec![(
                    0,
                    VNode::Element {
                        tag: Tag::Empty,
                        attributes: HashMap::with_capacity(0),
                        events: Events(HashMap::with_capacity(0)),
                        children: vec![element],
                    },
                )],
                patches: Vec::default(),
                last_deep: 0,
                current_node_id: root_node,
                last_parent: root_node,
            },
            document: Some(web_sys::window().unwrap().document().unwrap()),
        }));

        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        *g.borrow_mut() = Some(Closure::wrap(
            Box::new(move |deadline: web_sys::IdleDeadline| {
                APP.with(|app| {
                    if let Ok(mut app) = app.try_borrow_mut() {
                        if let Some(app) = &mut *app {
                            if !app.working_context.working_complete() {
                                work_on_vdom(app, || deadline.time_remaining() > 1.0);
                                if deadline.time_remaining() > 1.0
                                    && app.working_context.working_complete()
                                {
                                    commit_patches(app);
                                }
                            }
                        }
                    }
                    request_idle_callback(f.borrow().as_ref().unwrap());
                });
            }) as Box<dyn FnMut(web_sys::IdleDeadline)>,
        ));

        request_idle_callback(g.borrow().as_ref().unwrap());
    });
}
