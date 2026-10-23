// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

#![doc = include_str!("../README.md")]
#![deny(unsafe_op_in_unsafe_fn)]

mod adapter;
mod context;
mod event;
mod ffi;
mod filters;
mod node;
mod object;
mod patch;
mod subclass;
mod util;

pub use adapter::Adapter;
pub use event::QueuedEvents;
pub use patch::add_focus_forwarder_to_window_class;
pub use subclass::SubclassingAdapter;
