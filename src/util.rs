// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit_consumer::{NodeRef as Node, TextPosition, TextRange};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    pub location: usize,
    pub length: usize,
}

pub fn from_ns_range<'a>(node: &'a Node<'a>, range: Range) -> Option<TextRange<'a>> {
    let pos = node.text_position_from_global_utf16_index(range.location)?;
    let mut text_range = pos.to_degenerate_range();
    if range.length > 0 {
        let end = node.text_position_from_global_utf16_index(range.location.checked_add(range.length)?)?;
        text_range.set_end(end);
    }
    Some(text_range)
}

pub fn to_ns_range(range: &TextRange) -> Range {
    let start = range.start().to_global_utf16_index();
    let end = range.end().to_global_utf16_index();
    Range {
        location: start,
        length: end.saturating_sub(start),
    }
}

pub fn to_ns_range_for_character(pos: &TextPosition) -> Range {
    let mut range = pos.to_degenerate_range();
    if !pos.is_document_end() {
        range.set_end(pos.forward_to_character_end());
    }
    to_ns_range(&range)
}
