// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::collections::{HashSet, VecDeque};
use std::rc::Rc;

use accesskit::{Live, Role};
use accesskit_consumer::{FilterResult, Node, NodeId, TreeChangeHandler};

use crate::context::Context;
use crate::ffi;
use crate::filters::filter;
use crate::node::{NodeWrapper, Value};
use crate::object::Object;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notification {
    FocusedUiElementChanged = 0,
    ValueChanged = 1,
    SelectedTextChanged = 2,
    TitleChanged = 3,
    SelectedRowsChanged = 4,
    UiElementDestroyed = 5,
}

impl Notification {
    pub fn post(self, element: &Object) {
        unsafe { ffi::accesskit_rs_post(element.as_ptr(), self as i32) };
    }
}

pub enum QueuedEvent {
    Generic { node_id: NodeId, notification: Notification },
    NodeDestroyed(NodeId),
    Announcement { text: String, assertive: bool },
}

impl QueuedEvent {
    fn live_region_announcement(node: &Node) -> Self {
        Self::Announcement {
            text: node.value().unwrap_or_default(),
            assertive: node.live() == Live::Assertive,
        }
    }

    fn raise(self, context: &Rc<Context>) {
        match self {
            Self::Generic { node_id, notification } => {
                notification.post(&context.get_or_create_platform_node(node_id));
            }
            Self::NodeDestroyed(node_id) => {
                if let Some(platform_node) = context.remove_platform_node(node_id) {
                    Notification::UiElementDestroyed.post(&platform_node);
                }
            }
            Self::Announcement { text, assertive } => unsafe {
                ffi::accesskit_rs_announce(context.view.as_ptr(), text.as_ptr(), text.len(), assertive);
            },
        }
    }
}

#[must_use = "events must be explicitly raised"]
pub struct QueuedEvents {
    context: Rc<Context>,
    events: Vec<QueuedEvent>,
}

impl QueuedEvents {
    pub(crate) const fn new(context: Rc<Context>, events: Vec<QueuedEvent>) -> Self {
        Self { context, events }
    }

    pub fn raise(self) {
        for event in self.events {
            event.raise(&self.context);
        }
    }
}

pub const fn focus_event(node_id: NodeId) -> QueuedEvent {
    QueuedEvent::Generic {
        node_id,
        notification: Notification::FocusedUiElementChanged,
    }
}

pub struct EventGenerator {
    context: Rc<Context>,
    events: Vec<QueuedEvent>,
    text_changed: HashSet<NodeId>,
    selected_rows_changed: HashSet<NodeId>,
}

impl EventGenerator {
    pub fn new(context: Rc<Context>) -> Self {
        Self {
            context,
            events: Vec::new(),
            text_changed: HashSet::new(),
            selected_rows_changed: HashSet::new(),
        }
    }

    pub fn into_result(self) -> QueuedEvents {
        QueuedEvents::new(self.context, self.events)
    }

    fn remove_subtree(&mut self, node: &Node) {
        let mut to_remove = VecDeque::new();
        to_remove.push_back(*node);
        while let Some(node) = to_remove.pop_front() {
            for child in node.filtered_children(&filter) {
                to_remove.push_back(child);
            }
            self.events.push(QueuedEvent::NodeDestroyed(node.id()));
        }
    }

    fn insert_text_change_if_needed_parent(&mut self, node: &Node) {
        if !node.supports_text_ranges() {
            return;
        }
        let id = node.id();
        if self.text_changed.contains(&id) {
            return;
        }
        self.events.insert(
            0,
            QueuedEvent::Generic {
                node_id: id,
                notification: Notification::ValueChanged,
            },
        );
        self.text_changed.insert(id);
    }

    fn insert_text_change_if_needed(&mut self, node: &Node) {
        if node.role() != Role::TextRun {
            return;
        }
        if let Some(node) = node.filtered_parent(&filter) {
            self.insert_text_change_if_needed_parent(&node);
        }
    }

    fn enqueue_selected_rows_change_if_needed_parent(&mut self, node: &Node) {
        let id = node.id();
        if self.selected_rows_changed.contains(&id) {
            return;
        }
        self.events.push(QueuedEvent::Generic {
            node_id: id,
            notification: Notification::SelectedRowsChanged,
        });
        self.selected_rows_changed.insert(id);
    }

    fn enqueue_selected_rows_change_if_needed(&mut self, node: &Node) {
        if !NodeWrapper(node).is_item_like() {
            return;
        }
        if let Some(node) = node.selection_container(&filter) {
            self.enqueue_selected_rows_change_if_needed_parent(&node);
        }
    }
}

impl TreeChangeHandler for EventGenerator {
    fn node_added(&mut self, node: &Node) {
        self.insert_text_change_if_needed(node);
        if filter(node) != FilterResult::Include {
            return;
        }
        if node.is_selected() == Some(true) {
            self.enqueue_selected_rows_change_if_needed(node);
        }
        if node.value().is_some() && node.live() != Live::Off {
            self.events.push(QueuedEvent::live_region_announcement(node));
        }
    }

    fn node_updated(&mut self, old_node: &Node, new_node: &Node) {
        if old_node.raw_value() != new_node.raw_value() {
            self.insert_text_change_if_needed(new_node);
        }
        let old_filter_result = filter(old_node);
        let new_filter_result = filter(new_node);
        if new_filter_result != FilterResult::Include {
            if old_filter_result == FilterResult::Include && old_node.is_selected() == Some(true) {
                self.enqueue_selected_rows_change_if_needed(old_node);
            }
            if new_filter_result == FilterResult::ExcludeSubtree {
                self.remove_subtree(old_node);
            } else {
                self.events.push(QueuedEvent::NodeDestroyed(new_node.id()));
            }
            return;
        }
        let node_id = new_node.id();
        let old_wrapper = NodeWrapper(old_node);
        let new_wrapper = NodeWrapper(new_node);
        if old_wrapper.title() != new_wrapper.title() {
            self.events.push(QueuedEvent::Generic {
                node_id,
                notification: Notification::TitleChanged,
            });
        }
        let new_value = new_wrapper.value();
        if old_wrapper.value() != new_value {
            let event = QueuedEvent::Generic {
                node_id,
                notification: Notification::ValueChanged,
            };
            if !new_node.is_focused() && matches!(new_value, Some(Value::Bool(_))) {
                self.events.insert(0, event);
            } else {
                self.events.push(event);
            }
        }
        if old_wrapper.supports_text_ranges()
            && new_wrapper.supports_text_ranges()
            && old_wrapper.raw_text_selection() != new_wrapper.raw_text_selection()
        {
            self.events.push(QueuedEvent::Generic {
                node_id,
                notification: Notification::SelectedTextChanged,
            });
        }
        if new_node.value().is_some()
            && new_node.live() != Live::Off
            && (new_node.value() != old_node.value()
                || new_node.live() != old_node.live()
                || old_filter_result != FilterResult::Include)
        {
            self.events.push(QueuedEvent::live_region_announcement(new_node));
        }
        if new_node.is_selected() != old_node.is_selected()
            || (old_filter_result != FilterResult::Include && new_node.is_selected() == Some(true))
        {
            self.enqueue_selected_rows_change_if_needed(new_node);
        }
    }

    fn focus_moved(&mut self, _old_node: Option<&Node>, new_node: Option<&Node>) {
        if let Some(new_node) = new_node {
            if filter(new_node) != FilterResult::Include {
                return;
            }
            self.events.push(focus_event(new_node.id()));
        }
    }

    fn node_removed(&mut self, node: &Node) {
        self.insert_text_change_if_needed(node);
        if node.is_selected() == Some(true) {
            self.enqueue_selected_rows_change_if_needed(node);
        }
        self.events.push(QueuedEvent::NodeDestroyed(node.id()));
    }
}
