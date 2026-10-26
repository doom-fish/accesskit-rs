// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use accesskit::{ActionHandler, ActionRequest};
use accesskit_consumer::{FullNodeId as NodeId, Tree};

use crate::event::Notification;
use crate::ffi;
use crate::node::NodeRef;
use crate::object::Object;

pub trait ActionHandlerNoMut {
    fn do_action(&self, request: ActionRequest);
}

pub struct ActionHandlerWrapper<H: ActionHandler>(RefCell<H>);

impl<H: 'static + ActionHandler> ActionHandlerWrapper<H> {
    pub const fn new(inner: H) -> Self {
        Self(RefCell::new(inner))
    }
}

impl<H: ActionHandler> ActionHandlerNoMut for ActionHandlerWrapper<H> {
    fn do_action(&self, request: ActionRequest) {
        self.0.borrow_mut().do_action(request);
    }
}

pub struct Context {
    pub view: Object,
    pub tree: RefCell<Tree>,
    pub action_handler: Rc<dyn ActionHandlerNoMut>,
    platform_nodes: RefCell<HashMap<NodeId, Object>>,
}

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Context")
            .field("view", &self.view)
            .field("tree", &self.tree)
            .field("action_handler", &"ActionHandler")
            .field("platform_nodes", &self.platform_nodes)
            .finish()
    }
}

impl Context {
    pub fn new(view: Object, tree: Tree, action_handler: Rc<dyn ActionHandlerNoMut>) -> Rc<Self> {
        Rc::new(Self {
            view,
            tree: RefCell::new(tree),
            action_handler,
            platform_nodes: RefCell::new(HashMap::new()),
        })
    }

    pub fn get_or_create_platform_node(self: &Rc<Self>, id: NodeId) -> Object {
        let mut platform_nodes = self.platform_nodes.borrow_mut();
        if let Some(result) = platform_nodes.get(&id) {
            return result.clone();
        }
        let node = Box::into_raw(Box::new(NodeRef {
            context: Rc::downgrade(self),
            id,
        }));
        let result = unsafe { Object::from_retained(ffi::accesskit_rs_node_new(self.view.as_ptr(), node.cast())) }
            .expect("the bridge gives a node");
        platform_nodes.insert(id, result.clone());
        result
    }

    pub fn remove_platform_node(&self, id: NodeId) -> Option<Object> {
        self.platform_nodes.borrow_mut().remove(&id)
    }

    pub fn do_action(&self, request: ActionRequest) {
        self.action_handler.do_action(request);
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        for platform_node in self.platform_nodes.borrow().values() {
            Notification::UiElementDestroyed.post(platform_node);
        }
    }
}
