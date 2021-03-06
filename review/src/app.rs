use crate::fiber::{FiberId, FiberTree};
use crate::node::{Element, Node};
use crate::reconciliation::{commit, perform_unit_of_work};
use crate::request_animation_frame;
use crate::{Events, Tag, VNode};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;

#[derive(Debug)]
pub(crate) struct App {
    pub fiber_tree: FiberTree,
    pub wip_root: Option<FiberId>,
    pub next_unit_of_work: Option<FiberId>,
    pub document: Option<web_sys::Document>,
}

thread_local! {
    pub(crate) static APP: RefCell<Option<App>> = RefCell::new(None);
}

/// Starts a reView app mounted to the element with the specified id.
///
/// # Example
/// ```rust,no_run
/// # use review::Tag::Div;
/// review::render(Div.into(), "root");
/// ```
pub fn render(element: VNode, container_id: &str) {
    let root_dom = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id(container_id)
        .expect("error during root container retrival");

    APP.with(|app| {
        let mut fiber_tree = FiberTree::default();
        let root_id = fiber_tree.new_node(Node::Element(Element {
            dom: Some(root_dom),
            tag: Tag::Empty,
            attributes: HashMap::with_capacity(0),
            events: Events(HashMap::with_capacity(0)),
            unprocessed_children: vec![element],
        }));
        app.replace(Some(App {
            fiber_tree,
            next_unit_of_work: Some(root_id),
            wip_root: Some(root_id),
            document: Some(web_sys::window().unwrap().document().unwrap()),
        }));

        let run_for = instant::Duration::from_millis(5);

        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            APP.with(|app| {
                if let Ok(mut app) = app.try_borrow_mut() {
                    if let Some(app) = &mut *app {
                        let deadline = instant::Instant::now();
                        let check_deadline = || deadline.elapsed() < run_for;
                        work_loop(app, check_deadline);

                        commit_work(app, check_deadline);
                    }
                }
                request_animation_frame(f.borrow().as_ref().unwrap());
            });
        }) as Box<dyn FnMut()>));

        request_animation_frame(g.borrow().as_ref().unwrap());
    });
}

pub(crate) fn work_loop<F: Fn() -> bool>(app: &mut App, continue_working: F) {
    while app.next_unit_of_work.is_some() && continue_working() {
        if let Some(current_id) = app.next_unit_of_work {
            app.next_unit_of_work =
                perform_unit_of_work(current_id, &mut app.fiber_tree, app.document.as_ref());
        }
    }
}

pub(crate) fn commit_work<F: Fn() -> bool>(app: &mut App, continue_working: F) {
    if let Some(wip_root) = app.wip_root {
        if continue_working() {
            if let Some(child_id) = app
                .fiber_tree
                .get(wip_root)
                .and_then(|wip_node| wip_node.child)
            {
                commit(Some(child_id), &mut app.fiber_tree);
                app.wip_root = None;
            }
        }
    }
}
