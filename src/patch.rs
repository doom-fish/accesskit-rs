// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::ffi::CString;

use crate::ffi;

pub unsafe fn add_focus_forwarder_to_window_class(class_name: &str) {
    let name = CString::new(class_name).expect("a class name has no NUL byte");
    match unsafe { ffi::accesskit_rs_add_focus_forwarder(name.as_ptr()) } {
        0 => {}
        1 => panic!("there is no class with the name {class_name}"),
        _ => panic!("could not add accessibilityFocusedUIElement to {class_name}"),
    }
}
