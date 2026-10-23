// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use core::ffi::{c_int, c_void};
use core::ptr;
use std::fmt::{self, Debug, Formatter};
use std::rc::Rc;
use std::sync::Once;

use accesskit::{
    ActionHandler, ActionRequest, ActivationHandler, Node as NodeProvider, NodeId as LocalNodeId, Point, Role,
    Tree as TreeData, TreeId, TreeUpdate,
};
use accesskit_consumer::{FilterResult, Tree};

use crate::context::{ActionHandlerNoMut, ActionHandlerWrapper, Context};
use crate::event::{focus_event, EventGenerator, QueuedEvents};
use crate::ffi;
use crate::filters::filter;
use crate::node::{self, can_be_focused};
use crate::object::Object;
use crate::subclass;

const PLACEHOLDER_ROOT_ID: LocalNodeId = LocalNodeId(0);

extern "C" {
    fn pthread_main_np() -> c_int;
}

static INSTALL: Once = Once::new();

fn install_callbacks() {
    INSTALL.call_once(|| unsafe {
        ffi::accesskit_rs_install(
            node::node_string,
            node::node_number,
            node::node_integer,
            node::node_flag,
            node::node_value,
            node::node_list,
            node::node_rect,
            node::node_range,
            node::node_text_runs,
            node::node_action,
            node::node_release,
            subclass::view_children,
            subclass::view_focus,
            subclass::view_hit_test,
        );
    });
}

enum State {
    Inactive {
        view: Object,
        is_view_focused: bool,
        action_handler: Rc<dyn ActionHandlerNoMut>,
    },
    Placeholder {
        placeholder_context: Rc<Context>,
        is_view_focused: bool,
        action_handler: Rc<dyn ActionHandlerNoMut>,
    },
    Active(Rc<Context>),
}

impl Debug for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inactive {
                view, is_view_focused, ..
            } => f
                .debug_struct("Inactive")
                .field("view", view)
                .field("is_view_focused", is_view_focused)
                .finish_non_exhaustive(),
            Self::Placeholder {
                placeholder_context,
                is_view_focused,
                ..
            } => f
                .debug_struct("Placeholder")
                .field("placeholder_context", placeholder_context)
                .field("is_view_focused", is_view_focused)
                .finish_non_exhaustive(),
            Self::Active(context) => f.debug_struct("Active").field("context", context).finish(),
        }
    }
}

struct PlaceholderActionHandler;

impl ActionHandler for PlaceholderActionHandler {
    fn do_action(&mut self, _request: ActionRequest) {}
}

#[derive(Debug)]
pub struct Adapter {
    state: State,
}

impl Adapter {
    pub unsafe fn new(view: *mut c_void, is_view_focused: bool, action_handler: impl 'static + ActionHandler) -> Self {
        assert!(
            unsafe { pthread_main_np() } != 0,
            "an AccessKit adapter must start on the main thread"
        );
        assert!(!view.is_null(), "an AccessKit adapter needs a view");
        install_callbacks();
        let view = unsafe { Object::from_retained(ffi::accesskit_rs_view_ref_new(view)) }
            .expect("the bridge gives a view reference");
        Self {
            state: State::Inactive {
                view,
                is_view_focused,
                action_handler: Rc::new(ActionHandlerWrapper::new(action_handler)),
            },
        }
    }

    pub fn update_if_active(&mut self, update_factory: impl FnOnce() -> TreeUpdate) -> Option<QueuedEvents> {
        match &self.state {
            State::Inactive { .. } => None,
            State::Placeholder {
                placeholder_context,
                is_view_focused,
                action_handler,
            } => {
                let tree = Tree::new(update_factory(), *is_view_focused);
                let context = Context::new(placeholder_context.view.clone(), tree, Rc::clone(action_handler));
                let result = context
                    .tree
                    .borrow()
                    .state()
                    .focus()
                    .map(|node| QueuedEvents::new(Rc::clone(&context), vec![focus_event(node.id())]));
                self.state = State::Active(context);
                result
            }
            State::Active(context) => {
                let mut event_generator = EventGenerator::new(Rc::clone(context));
                context
                    .tree
                    .borrow_mut()
                    .update_and_process_changes(update_factory(), &mut event_generator);
                Some(event_generator.into_result())
            }
        }
    }

    pub fn update_view_focus_state(&mut self, is_focused: bool) -> Option<QueuedEvents> {
        match &mut self.state {
            State::Inactive { is_view_focused, .. } | State::Placeholder { is_view_focused, .. } => {
                *is_view_focused = is_focused;
                None
            }
            State::Active(context) => {
                let mut event_generator = EventGenerator::new(Rc::clone(context));
                context
                    .tree
                    .borrow_mut()
                    .update_host_focus_state_and_process_changes(is_focused, &mut event_generator);
                Some(event_generator.into_result())
            }
        }
    }

    fn get_or_init_context<H: ActivationHandler + ?Sized>(&mut self, activation_handler: &mut H) -> Rc<Context> {
        match &self.state {
            State::Inactive {
                view,
                is_view_focused,
                action_handler,
            } => {
                if let Some(initial_state) = activation_handler.request_initial_tree() {
                    let tree = Tree::new(initial_state, *is_view_focused);
                    let context = Context::new(view.clone(), tree, Rc::clone(action_handler));
                    self.state = State::Active(Rc::clone(&context));
                    context
                } else {
                    let placeholder_update = TreeUpdate {
                        nodes: vec![(PLACEHOLDER_ROOT_ID, NodeProvider::new(Role::Window))],
                        tree: Some(TreeData::new(PLACEHOLDER_ROOT_ID)),
                        tree_id: TreeId::ROOT,
                        focus: PLACEHOLDER_ROOT_ID,
                    };
                    let placeholder_tree = Tree::new(placeholder_update, false);
                    let placeholder_context = Context::new(
                        view.clone(),
                        placeholder_tree,
                        Rc::new(ActionHandlerWrapper::new(PlaceholderActionHandler)),
                    );
                    self.state = State::Placeholder {
                        placeholder_context: Rc::clone(&placeholder_context),
                        is_view_focused: *is_view_focused,
                        action_handler: Rc::clone(action_handler),
                    };
                    placeholder_context
                }
            }
            State::Placeholder {
                placeholder_context, ..
            } => Rc::clone(placeholder_context),
            State::Active(context) => Rc::clone(context),
        }
    }

    fn view(&self) -> &Object {
        match &self.state {
            State::Inactive { view, .. } => view,
            State::Placeholder {
                placeholder_context, ..
            } => &placeholder_context.view,
            State::Active(context) => &context.view,
        }
    }

    pub(crate) fn child_nodes<H: ActivationHandler + ?Sized>(&mut self, activation_handler: &mut H) -> Vec<Object> {
        let context = self.get_or_init_context(activation_handler);
        let tree = context.tree.borrow();
        let state = tree.state();
        let node = state.root();
        let nodes = if filter(&node) == FilterResult::Include {
            vec![context.get_or_create_platform_node(node.id())]
        } else {
            node.filtered_children(filter)
                .map(|node| context.get_or_create_platform_node(node.id()))
                .collect()
        };
        nodes
    }

    pub(crate) fn focus_node<H: ActivationHandler + ?Sized>(&mut self, activation_handler: &mut H) -> Option<Object> {
        let context = self.get_or_init_context(activation_handler);
        let tree = context.tree.borrow();
        let node = tree.state().focus().filter(can_be_focused);
        node.map(|node| context.get_or_create_platform_node(node.id()))
    }

    pub(crate) fn hit_test_node<H: ActivationHandler + ?Sized>(
        &mut self,
        point: Point,
        activation_handler: &mut H,
    ) -> Object {
        let context = self.get_or_init_context(activation_handler);
        let tree = context.tree.borrow();
        let state = tree.state();
        let root = state.root();
        let point = root.transform().inverse() * point;
        let node = root.node_at_point(point, &filter).unwrap_or(root);
        context.get_or_create_platform_node(node.id())
    }

    pub fn view_children<H: ActivationHandler + ?Sized>(&mut self, activation_handler: &mut H) -> *mut c_void {
        let nodes = self.child_nodes(activation_handler);
        let pointers: Vec<*mut c_void> = nodes.iter().map(Object::as_ptr).collect();
        unsafe { ffi::accesskit_rs_array(pointers.as_ptr(), pointers.len()) }
    }

    pub fn focus<H: ActivationHandler + ?Sized>(&mut self, activation_handler: &mut H) -> *mut c_void {
        self.focus_node(activation_handler)
            .map_or(ptr::null_mut(), |node| unsafe { ffi::accesskit_rs_autorelease(node.as_ptr()) })
    }

    pub fn hit_test<H: ActivationHandler + ?Sized>(&mut self, x: f64, y: f64, activation_handler: &mut H) -> *mut c_void {
        let mut pixel = [0.0; 2];
        if !unsafe { ffi::accesskit_rs_view_ref_pixel_point(self.view().as_ptr(), x, y, pixel.as_mut_ptr()) } {
            return ptr::null_mut();
        }
        let node = self.hit_test_node(Point::new(pixel[0], pixel[1]), activation_handler);
        unsafe { ffi::accesskit_rs_autorelease(node.as_ptr()) }
    }
}
