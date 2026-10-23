// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use core::ffi::c_void;
use core::ptr;
use std::cell::RefCell;

use accesskit::{ActionHandler, ActivationHandler, Point, TreeUpdate};

use crate::adapter::Adapter;
use crate::event::QueuedEvents;
use crate::ffi;
use crate::node::give_list;
use crate::object::Object;

struct AttachedState {
    adapter: Adapter,
    activation_handler: Box<dyn ActivationHandler>,
}

pub struct SubclassingAdapter {
    view: Object,
    attachment: Object,
    state: Box<RefCell<AttachedState>>,
}

impl SubclassingAdapter {
    pub unsafe fn new(
        view: *mut c_void,
        activation_handler: impl 'static + ActivationHandler,
        action_handler: impl 'static + ActionHandler,
    ) -> Self {
        let view = unsafe { Object::retain(view) }.expect("a subclassing adapter needs a view");
        Self::attach(view, activation_handler, action_handler)
    }

    pub unsafe fn for_window(
        window: *mut c_void,
        activation_handler: impl 'static + ActivationHandler,
        action_handler: impl 'static + ActionHandler,
    ) -> Self {
        assert!(!window.is_null(), "a subclassing adapter needs a window");
        let view = unsafe { Object::from_retained(ffi::accesskit_rs_window_content_view(window)) }
            .expect("the window has a content view");
        Self::attach(view, activation_handler, action_handler)
    }

    fn attach(
        view: Object,
        activation_handler: impl 'static + ActivationHandler,
        action_handler: impl 'static + ActionHandler,
    ) -> Self {
        let adapter = unsafe { Adapter::new(view.as_ptr(), false, action_handler) };
        let state = Box::new(RefCell::new(AttachedState {
            adapter,
            activation_handler: Box::new(activation_handler),
        }));
        let state_pointer = ptr::from_ref::<RefCell<AttachedState>>(&state).cast_mut().cast::<c_void>();
        let attachment = unsafe { Object::from_retained(ffi::accesskit_rs_subclass_attach(view.as_ptr(), state_pointer)) }
            .unwrap_or_else(|| panic!("subclassing adapter already instantiated on view {:?}", view.as_ptr()));
        Self {
            view,
            attachment,
            state,
        }
    }

    pub fn update_if_active(&mut self, update_factory: impl FnOnce() -> TreeUpdate) -> Option<QueuedEvents> {
        self.state.borrow_mut().adapter.update_if_active(update_factory)
    }

    pub fn update_view_focus_state(&mut self, is_focused: bool) -> Option<QueuedEvents> {
        self.state.borrow_mut().adapter.update_view_focus_state(is_focused)
    }
}

impl Drop for SubclassingAdapter {
    fn drop(&mut self) {
        unsafe { ffi::accesskit_rs_subclass_detach(self.view.as_ptr(), self.attachment.as_ptr()) };
    }
}

fn with_state<T>(state: *mut c_void, f: impl FnOnce(&mut Adapter, &mut dyn ActivationHandler) -> T) -> T {
    let state = unsafe { &*state.cast::<RefCell<AttachedState>>() };
    let mut state = state.borrow_mut();
    let AttachedState {
        adapter,
        activation_handler,
    } = &mut *state;
    f(adapter, activation_handler.as_mut())
}

pub unsafe extern "C" fn view_children(state: *mut c_void, out_ptr: *mut *const *mut c_void, out_len: *mut usize) {
    let nodes = with_state(state, |adapter, activation_handler| adapter.child_nodes(activation_handler));
    unsafe { give_list(&nodes, out_ptr, out_len) };
}

pub unsafe extern "C" fn view_focus(state: *mut c_void) -> *mut c_void {
    with_state(state, |adapter, activation_handler| adapter.focus_node(activation_handler))
        .map_or(ptr::null_mut(), |node| node.as_ptr())
}

pub unsafe extern "C" fn view_hit_test(state: *mut c_void, x: f64, y: f64) -> *mut c_void {
    with_state(state, |adapter, activation_handler| {
        adapter.hit_test_node(Point::new(x, y), activation_handler)
    })
    .as_ptr()
}
