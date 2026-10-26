// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from Chromium's accessibility abstraction.
// Copyright 2018 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

use core::ffi::c_void;
use core::ptr;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use accesskit::{Action, ActionData, ActionRequest, Color, Orientation, Point, Rect, Role, TextAlign, TextSelection, Toggled};
use accesskit_consumer::{FilterResult, FullNodeId as NodeId, NodeRef as Node, Tree};

use crate::context::Context;
use crate::ffi;
use crate::filters::filter;
use crate::object::Object;
use crate::util::{from_ns_range, to_ns_range, to_ns_range_for_character, Range};

const UNKNOWN: &str = "AXUnknown";

mod query {
    pub const ROLE: i32 = 0;
    pub const SUBROLE: i32 = 1;
    pub const ROLE_DESCRIPTION: i32 = 2;
    pub const IDENTIFIER: i32 = 3;
    pub const TITLE: i32 = 4;
    pub const HELP: i32 = 5;
    pub const PLACEHOLDER: i32 = 6;
    pub const URL: i32 = 7;
    pub const SELECTED_TEXT: i32 = 8;
    pub const STRING_FOR_RANGE: i32 = 9;
    pub const BRAILLE_LABEL: i32 = 10;
    pub const BRAILLE_ROLE_DESCRIPTION: i32 = 11;

    pub const MIN_VALUE: i32 = 0;
    pub const MAX_VALUE: i32 = 1;

    pub const ORIENTATION: i32 = 0;
    pub const NUMBER_OF_CHARACTERS: i32 = 1;
    pub const INSERTION_POINT_LINE_NUMBER: i32 = 2;
    pub const LINE_FOR_INDEX: i32 = 3;

    pub const IS_ELEMENT: i32 = 0;
    pub const IS_FOCUSED: i32 = 1;
    pub const IS_ENABLED: i32 = 2;
    pub const IS_REQUIRED: i32 = 3;
    pub const IS_SELECTED: i32 = 4;
    pub const IS_MODAL: i32 = 5;
    pub const SELECTOR_ALLOWED: i32 = 6;
    pub const SUPPORTS_SCROLL_TO_VISIBLE: i32 = 7;

    pub const CHILDREN: i32 = 0;
    pub const CHILDREN_IN_NAVIGATION_ORDER: i32 = 1;
    pub const SELECTED_CHILDREN: i32 = 2;
    pub const ROWS: i32 = 3;
    pub const SELECTED_ROWS: i32 = 4;
    pub const LINKED: i32 = 5;
    pub const TABS: i32 = 6;
    pub const PARENT: i32 = 7;
    pub const WINDOW: i32 = 8;
    pub const TOP_LEVEL: i32 = 9;

    pub const FRAME: i32 = 0;
    pub const FRAME_FOR_RANGE: i32 = 1;

    pub const SELECTED_TEXT_RANGE: i32 = 0;
    pub const RANGE_FOR_LINE: i32 = 1;
    pub const RANGE_FOR_POSITION: i32 = 2;
    pub const RANGE_FOR_INDEX: i32 = 3;
    pub const STYLE_RANGE_FOR_INDEX: i32 = 4;

    pub const SET_VALUE_STRING: i32 = 0;
    pub const SET_VALUE_NUMBER: i32 = 1;
    pub const SET_FOCUSED: i32 = 2;
    pub const PRESS: i32 = 3;
    pub const INCREMENT: i32 = 4;
    pub const DECREMENT: i32 = 5;
    pub const SET_SELECTED_TEXT_RANGE: i32 = 6;
    pub const SET_SELECTED: i32 = 7;
    pub const PICK: i32 = 8;
    pub const SCROLL_TO_VISIBLE: i32 = 9;
}

#[allow(clippy::match_same_arms, clippy::too_many_lines)]
fn ns_role(node: &Node) -> &'static str {
    match node.role() {
        Role::Unknown => "AXUnknown",
        Role::TextRun => "AXUnknown",
        Role::Cell | Role::GridCell => "AXCell",
        Role::Label => "AXStaticText",
        Role::Image => "AXImage",
        Role::Link => "AXLink",
        Role::Row => "AXRow",
        Role::ListItem => "AXGroup",
        Role::ListMarker => "AXListMarker",
        Role::TreeItem => "AXRow",
        Role::ListBoxOption => "AXStaticText",
        Role::MenuItem => "AXMenuItem",
        Role::MenuListOption => "AXMenuItem",
        Role::Paragraph => "AXGroup",
        Role::GenericContainer => "AXUnknown",
        Role::CheckBox => "AXCheckBox",
        Role::RadioButton => "AXRadioButton",
        Role::TextInput
        | Role::SearchInput
        | Role::EmailInput
        | Role::NumberInput
        | Role::PasswordInput
        | Role::PhoneNumberInput
        | Role::UrlInput => "AXTextField",
        Role::Button => {
            if node.toggled().is_some() {
                "AXCheckBox"
            } else {
                "AXButton"
            }
        }
        Role::DefaultButton => "AXButton",
        Role::Pane => "AXUnknown",
        Role::RowHeader => "AXCell",
        Role::ColumnHeader => "AXCell",
        Role::RowGroup => "AXGroup",
        Role::List => "AXList",
        Role::Table => "AXTable",
        Role::LayoutTableCell => "AXGroup",
        Role::LayoutTableRow => "AXGroup",
        Role::LayoutTable => "AXGroup",
        Role::Switch => "AXCheckBox",
        Role::Menu => "AXMenu",
        Role::MultilineTextInput => "AXTextArea",
        Role::DateInput | Role::DateTimeInput | Role::WeekInput | Role::MonthInput => "AXDateField",
        Role::TimeInput => "AXTimeField",
        Role::Abbr => "AXGroup",
        Role::Alert => "AXGroup",
        Role::AlertDialog => "AXWindow",
        Role::Application => "AXGroup",
        Role::Article => "AXGroup",
        Role::Audio => "AXGroup",
        Role::Banner => "AXGroup",
        Role::Blockquote => "AXGroup",
        Role::Canvas => "AXImage",
        Role::Caption => "AXGroup",
        Role::Caret => "AXUnknown",
        Role::Code => "AXGroup",
        Role::ColorWell => "AXColorWell",
        Role::ComboBox => "AXPopUpButton",
        Role::EditableComboBox => "AXComboBox",
        Role::Complementary => "AXGroup",
        Role::Comment => "AXGroup",
        Role::ContentDeletion => "AXGroup",
        Role::ContentInsertion => "AXGroup",
        Role::ContentInfo => "AXGroup",
        Role::Definition => "AXGroup",
        Role::DescriptionList => "AXList",
        Role::Details => "AXGroup",
        Role::Dialog => "AXWindow",
        Role::DisclosureTriangle => "AXButton",
        Role::Document => "AXGroup",
        Role::EmbeddedObject => "AXGroup",
        Role::Emphasis => "AXGroup",
        Role::Feed => "AXUnknown",
        Role::FigureCaption => "AXGroup",
        Role::Figure => "AXGroup",
        Role::Footer => "AXGroup",
        Role::Form => "AXGroup",
        Role::Grid => "AXTable",
        Role::Group => "AXGroup",
        Role::Header => "AXGroup",
        Role::Heading => "AXHeading",
        Role::Iframe => "AXGroup",
        Role::IframePresentational => "AXGroup",
        Role::ImeCandidate => "AXUnknown",
        Role::Keyboard => "AXUnknown",
        Role::Legend => "AXGroup",
        Role::LineBreak => "AXGroup",
        Role::ListBox => "AXList",
        Role::Log => "AXGroup",
        Role::Main => "AXGroup",
        Role::Mark => "AXGroup",
        Role::Marquee => "AXGroup",
        Role::Math => "AXGroup",
        Role::MenuBar => "AXMenuBar",
        Role::MenuItemCheckBox => "AXMenuItem",
        Role::MenuItemRadio => "AXMenuItem",
        Role::MenuListPopup => "AXMenu",
        Role::Meter => "AXLevelIndicator",
        Role::Navigation => "AXGroup",
        Role::Note => "AXGroup",
        Role::PluginObject => "AXGroup",
        Role::ProgressIndicator => "AXProgressIndicator",
        Role::RadioGroup => "AXRadioGroup",
        Role::Region => "AXGroup",
        Role::RootWebArea => "AXWebArea",
        Role::Ruby => "AXGroup",
        Role::RubyAnnotation => "AXUnknown",
        Role::ScrollBar => "AXScrollBar",
        Role::ScrollView => "AXUnknown",
        Role::Search => "AXGroup",
        Role::Section => "AXGroup",
        Role::SectionFooter => "AXGroup",
        Role::SectionHeader => "AXGroup",
        Role::Slider => "AXSlider",
        Role::SpinButton => "AXIncrementor",
        Role::Splitter => "AXSplitter",
        Role::Status => "AXGroup",
        Role::Strong => "AXGroup",
        Role::Suggestion => "AXGroup",
        Role::SvgRoot => "AXGroup",
        Role::Tab => "AXRadioButton",
        Role::TabList => "AXTabGroup",
        Role::TabPanel => "AXGroup",
        Role::Term => "AXGroup",
        Role::Time => "AXGroup",
        Role::Timer => "AXGroup",
        Role::TitleBar => "AXStaticText",
        Role::Toolbar => "AXToolbar",
        Role::Tooltip => "AXGroup",
        Role::Tree => "AXOutline",
        Role::TreeGrid => "AXTable",
        Role::Video => "AXGroup",
        Role::WebView => "AXUnknown",
        Role::Window => "AXGroup",
        Role::PdfActionableHighlight => "AXButton",
        Role::PdfRoot => "AXGroup",
        Role::GraphicsDocument => "AXGroup",
        Role::GraphicsObject => "AXGroup",
        Role::GraphicsSymbol => "AXImage",
        Role::DocAbstract => "AXGroup",
        Role::DocAcknowledgements => "AXGroup",
        Role::DocAfterword => "AXGroup",
        Role::DocAppendix => "AXGroup",
        Role::DocBackLink => "AXLink",
        Role::DocBiblioEntry => "AXGroup",
        Role::DocBibliography => "AXGroup",
        Role::DocBiblioRef => "AXGroup",
        Role::DocChapter => "AXGroup",
        Role::DocColophon => "AXGroup",
        Role::DocConclusion => "AXGroup",
        Role::DocCover => "AXImage",
        Role::DocCredit => "AXGroup",
        Role::DocCredits => "AXGroup",
        Role::DocDedication => "AXGroup",
        Role::DocEndnote => "AXGroup",
        Role::DocEndnotes => "AXGroup",
        Role::DocEpigraph => "AXGroup",
        Role::DocEpilogue => "AXGroup",
        Role::DocErrata => "AXGroup",
        Role::DocExample => "AXGroup",
        Role::DocFootnote => "AXGroup",
        Role::DocForeword => "AXGroup",
        Role::DocGlossary => "AXGroup",
        Role::DocGlossRef => "AXLink",
        Role::DocIndex => "AXGroup",
        Role::DocIntroduction => "AXGroup",
        Role::DocNoteRef => "AXLink",
        Role::DocNotice => "AXGroup",
        Role::DocPageBreak => "AXSplitter",
        Role::DocPageFooter => "AXGroup",
        Role::DocPageHeader => "AXGroup",
        Role::DocPageList => "AXGroup",
        Role::DocPart => "AXGroup",
        Role::DocPreface => "AXGroup",
        Role::DocPrologue => "AXGroup",
        Role::DocPullquote => "AXGroup",
        Role::DocQna => "AXGroup",
        Role::DocSubtitle => "AXHeading",
        Role::DocTip => "AXGroup",
        Role::DocToc => "AXGroup",
        Role::ListGrid => "AXUnknown",
        Role::Terminal => "AXTextArea",
    }
}

fn ns_sub_role(node: &Node) -> &'static str {
    match node.role() {
        Role::Alert => "AXApplicationAlert",
        Role::AlertDialog | Role::Dialog => "AXDialog",
        Role::Article => "AXDocumentArticle",
        Role::Banner | Role::Header => "AXLandmarkBanner",
        Role::Button if node.toggled().is_some() => "AXToggle",
        Role::Code => "AXCodeStyleGroup",
        Role::Complementary => "AXLandmarkComplementary",
        Role::ContentDeletion => "AXDeleteStyleGroup",
        Role::ContentInsertion => "AXInsertStyleGroup",
        Role::ContentInfo | Role::Footer => "AXLandmarkContentInfo",
        Role::Definition => "AXDefinition",
        Role::Document | Role::GraphicsDocument => "AXDocument",
        Role::Emphasis => "AXEmphasisStyleGroup",
        Role::Feed | Role::Group => "AXApplicationGroup",
        Role::Form => "AXLandmarkForm",
        Role::LayoutTableCell => "AXGroup",
        Role::LayoutTableRow => "AXTableRow",
        Role::Log => "AXApplicationLog",
        Role::Main => "AXLandmarkMain",
        Role::Marquee => "AXApplicationMarquee",
        Role::Math => "AXDocumentMath",
        Role::Meter => "AXMeter",
        Role::Navigation => "AXLandmarkNavigation",
        Role::Note => "AXDocumentNote",
        Role::PasswordInput => "AXSecureTextField",
        Role::Region => "AXLandmarkRegion",
        Role::Search => "AXLandmarkSearch",
        Role::SearchInput => "AXSearchField",
        Role::SectionFooter => "AXSectionFooter",
        Role::SectionHeader => "AXSectionHeader",
        Role::Status => "AXApplicationStatus",
        Role::Strong => "AXStrongStyleGroup",
        Role::Switch => "AXSwitch",
        Role::Tab => "AXTabButton",
        Role::TabPanel => "AXTabPanel",
        Role::Term => "AXTerm",
        Role::Time => "AXTimeGroup",
        Role::Timer => "AXApplicationTimer",
        Role::TreeItem => "AXOutlineRow",
        Role::Tooltip => "AXUserInterfaceTooltip",
        _ => UNKNOWN,
    }
}

pub fn can_be_focused(node: &Node) -> bool {
    filter(node) == FilterResult::Include && node.role() != Role::Window
}

#[derive(PartialEq)]
pub enum Value {
    Bool(bool),
    Number(f64),
    String(String),
}

pub struct NodeWrapper<'a>(pub &'a Node<'a>);

impl NodeWrapper<'_> {
    fn is_root(&self) -> bool {
        self.0.is_root()
    }

    pub fn title(&self) -> Option<String> {
        if self.is_root() && self.0.role() == Role::Window {
            return None;
        }
        self.0.label()
    }

    pub fn description(&self) -> Option<String> {
        self.0.description()
    }

    pub fn placeholder(&self) -> Option<&str> {
        self.0.placeholder()
    }

    pub fn value(&self) -> Option<Value> {
        if let Some(toggled) = self.0.toggled() {
            return Some(Value::Bool(toggled != Toggled::False));
        }
        if self.0.role() == Role::Tab {
            return Some(Value::Bool(self.0.is_selected().unwrap_or(false)));
        }
        if let Some(value) = self.0.value() {
            return Some(Value::String(value));
        }
        self.0.numeric_value().map(Value::Number)
    }

    pub fn supports_text_ranges(&self) -> bool {
        self.0.supports_text_ranges()
    }

    pub fn raw_text_selection(&self) -> Option<&TextSelection> {
        self.0.raw_text_selection()
    }

    fn is_container_with_selectable_children(&self) -> bool {
        self.0.is_container_with_selectable_children() && self.0.role() != Role::TabList
    }

    pub fn is_item_like(&self) -> bool {
        self.0.is_item_like() && self.0.role() != Role::Tab
    }
}

pub struct NodeRef {
    pub context: Weak<Context>,
    pub id: NodeId,
}

thread_local! {
    static TEXT: RefCell<String> = const { RefCell::new(String::new()) };
    static LIST: RefCell<Vec<*mut c_void>> = const { RefCell::new(Vec::new()) };
}

pub unsafe fn give_text(text: &str, out_ptr: *mut *const u8, out_len: *mut usize) {
    TEXT.with_borrow_mut(|scratch| {
        scratch.clear();
        scratch.push_str(text);
        unsafe {
            out_ptr.write(scratch.as_ptr());
            out_len.write(scratch.len());
        }
    });
}

pub unsafe fn give_list(nodes: &[Object], out_ptr: *mut *const *mut c_void, out_len: *mut usize) {
    LIST.with_borrow_mut(|scratch| {
        scratch.clear();
        scratch.extend(nodes.iter().map(Object::as_ptr));
        unsafe {
            out_ptr.write(scratch.as_ptr());
            out_len.write(scratch.len());
        }
    });
}

fn resolve_with_context<T>(node: *mut c_void, f: impl FnOnce(&Node, &Tree, &Rc<Context>) -> T) -> Option<T> {
    let node_ref = unsafe { &*node.cast::<NodeRef>() };
    let context = node_ref.context.upgrade()?;
    let tree = context.tree.borrow();
    let state = tree.state();
    let node = state.node_by_id(node_ref.id)?;
    let result = f(&node, &tree, &context);
    Some(result)
}

fn resolve<T>(node: *mut c_void, f: impl FnOnce(&Node) -> T) -> Option<T> {
    resolve_with_context(node, |node, _, _| f(node))
}

fn to_isize(value: usize) -> isize {
    isize::try_from(value).unwrap_or(isize::MAX)
}

enum Text {
    Nil,
    Given(String),
    Super,
}

fn optional_text(node: &Node, query: i32, range: Range) -> Option<String> {
    match query {
        query::IDENTIFIER => node.author_id().map(str::to_owned),
        query::TITLE => NodeWrapper(node).title(),
        query::HELP => NodeWrapper(node).description(),
        query::PLACEHOLDER => NodeWrapper(node).placeholder().map(str::to_owned),
        query::URL => node.supports_url().then(|| node.url()).flatten().map(str::to_owned),
        query::SELECTED_TEXT => node
            .supports_text_ranges()
            .then(|| node.text_selection())
            .flatten()
            .map(|range| range.text()),
        query::STRING_FOR_RANGE => node
            .supports_text_ranges()
            .then(|| from_ns_range(node, range))
            .flatten()
            .map(|range| range.text()),
        query::BRAILLE_LABEL => node
            .has_braille_label()
            .then(|| node.braille_label())
            .flatten()
            .map(str::to_owned),
        query::BRAILLE_ROLE_DESCRIPTION => node
            .has_braille_role_description()
            .then(|| node.braille_role_description())
            .flatten()
            .map(str::to_owned),
        _ => None,
    }
}

fn text(node: *mut c_void, query: i32, range: Range) -> Text {
    match query {
        query::ROLE => Text::Given(resolve(node, ns_role).unwrap_or(UNKNOWN).to_owned()),
        query::SUBROLE => Text::Given(resolve(node, ns_sub_role).unwrap_or(UNKNOWN).to_owned()),
        query::ROLE_DESCRIPTION => match resolve(node, |node| node.role_description().map(str::to_owned)) {
            None => Text::Nil,
            Some(None) => Text::Super,
            Some(Some(description)) => Text::Given(description),
        },
        _ => resolve(node, |node| optional_text(node, query, range))
            .flatten()
            .map_or(Text::Nil, Text::Given),
    }
}

pub unsafe extern "C" fn node_string(
    node: *mut c_void,
    query: i32,
    location: usize,
    length: usize,
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> i32 {
    match text(node, query, Range { location, length }) {
        Text::Nil => 0,
        Text::Given(text) => {
            unsafe { give_text(&text, out_ptr, out_len) };
            1
        }
        Text::Super => 2,
    }
}

pub unsafe extern "C" fn node_number(node: *mut c_void, query: i32, out: *mut f64) -> bool {
    let value = resolve(node, |node| match query {
        query::MIN_VALUE => node.min_numeric_value(),
        query::MAX_VALUE => node.max_numeric_value(),
        _ => None,
    })
    .flatten();
    value.is_some_and(|value| {
        unsafe { out.write(value) };
        true
    })
}

fn integer(node: &Node, query: i32, argument: isize) -> Option<isize> {
    let supports_text_ranges = node.supports_text_ranges();
    match query {
        query::ORIENTATION => Some(match node.orientation() {
            Some(Orientation::Horizontal) => 1,
            Some(Orientation::Vertical) => 2,
            None => 0,
        }),
        query::NUMBER_OF_CHARACTERS => Some(if supports_text_ranges {
            to_isize(node.document_range().end().to_global_utf16_index())
        } else {
            0
        }),
        query::INSERTION_POINT_LINE_NUMBER => Some(
            supports_text_ranges
                .then(|| node.text_selection_focus())
                .flatten()
                .map_or(0, |pos| to_isize(pos.to_line_index())),
        ),
        query::LINE_FOR_INDEX => Some(
            usize::try_from(argument)
                .ok()
                .filter(|_| supports_text_ranges)
                .and_then(|index| node.text_position_from_global_utf16_index(index))
                .map_or(0, |pos| to_isize(pos.to_line_index())),
        ),
        _ => None,
    }
}

pub unsafe extern "C" fn node_integer(node: *mut c_void, query: i32, argument: isize, out: *mut isize) -> bool {
    let value = resolve(node, |node| integer(node, query, argument)).flatten();
    value.is_some_and(|value| {
        unsafe { out.write(value) };
        true
    })
}

fn selector_allowed(node: &Node, group: i32) -> bool {
    let wrapper = NodeWrapper(node);
    match group {
        0 => true,
        1 => node.is_focusable(&filter),
        2 => node.is_clickable(&filter),
        3 => node.supports_increment(&filter),
        4 => node.supports_decrement(&filter),
        5 => node.supports_text_ranges(),
        6 => (node.supports_text_ranges() && !node.is_read_only()) || node.supports_action(Action::SetValue, &filter),
        7 => wrapper.is_item_like(),
        8 => wrapper.is_container_with_selectable_children(),
        9 => node.is_clickable(&filter) && wrapper.is_item_like() && node.is_selectable(),
        10 => node.role() == Role::TabList,
        11 => node.is_dialog(),
        12 => node.has_braille_label() || node.has_braille_role_description(),
        13 => node.supports_url(),
        _ => false,
    }
}

fn flag(node: &Node, query: i32, argument: i32) -> bool {
    match query {
        query::IS_ELEMENT => filter(node) == FilterResult::Include,
        query::IS_FOCUSED => node.is_focused() && can_be_focused(node),
        query::IS_ENABLED => !node.is_disabled(),
        query::IS_REQUIRED => node.is_required(),
        query::IS_SELECTED => {
            NodeWrapper(node).is_item_like() && node.is_selectable() && node.is_selected().unwrap_or(false)
        }
        query::IS_MODAL => node.is_modal(),
        query::SELECTOR_ALLOWED => selector_allowed(node, argument),
        query::SUPPORTS_SCROLL_TO_VISIBLE => node.supports_action(Action::ScrollIntoView, &filter),
        _ => false,
    }
}

pub unsafe extern "C" fn node_flag(node: *mut c_void, query: i32, argument: i32) -> bool {
    resolve(node, |node| flag(node, query, argument)).unwrap_or(false)
}

pub unsafe extern "C" fn node_value(
    node: *mut c_void,
    out_number: *mut f64,
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> i32 {
    match resolve(node, |node| NodeWrapper(node).value()).flatten() {
        None => 0,
        Some(Value::Bool(value)) => {
            unsafe { out_number.write(f64::from(u8::from(value))) };
            1
        }
        Some(Value::Number(value)) => {
            unsafe { out_number.write(value) };
            2
        }
        Some(Value::String(value)) => {
            unsafe { give_text(&value, out_ptr, out_len) };
            3
        }
    }
}

enum List {
    Nodes(Vec<Object>),
    ViewParent,
}

fn list(node: &Node, context: &Rc<Context>, query: i32) -> Option<List> {
    let wrapper = NodeWrapper(node);
    let platform_node = |node: Node| context.get_or_create_platform_node(node.id());
    match query {
        query::CHILDREN | query::CHILDREN_IN_NAVIGATION_ORDER => {
            Some(List::Nodes(node.filtered_children(filter).map(platform_node).collect()))
        }
        query::SELECTED_CHILDREN | query::SELECTED_ROWS => wrapper.is_container_with_selectable_children().then(|| {
            List::Nodes(
                node.items(filter)
                    .filter(|item| item.is_selected() == Some(true))
                    .map(platform_node)
                    .collect(),
            )
        }),
        query::ROWS => wrapper
            .is_container_with_selectable_children()
            .then(|| List::Nodes(node.items(filter).map(platform_node).collect())),
        query::LINKED => {
            let nodes: Vec<Object> = node
                .controls()
                .filter(|controlled| filter(controlled) == FilterResult::Include)
                .map(platform_node)
                .collect();
            (!nodes.is_empty()).then_some(List::Nodes(nodes))
        }
        query::TABS => (node.role() == Role::TabList).then(|| {
            List::Nodes(
                node.filtered_children(filter)
                    .filter(|child| child.role() == Role::Tab)
                    .map(platform_node)
                    .collect(),
            )
        }),
        query::PARENT => Some(
            node.filtered_parent(&filter)
                .map_or(List::ViewParent, |parent| List::Nodes(vec![platform_node(parent)])),
        ),
        query::WINDOW | query::TOP_LEVEL => Some(List::ViewParent),
        _ => None,
    }
}

pub unsafe extern "C" fn node_list(
    node: *mut c_void,
    query: i32,
    out_ptr: *mut *const *mut c_void,
    out_len: *mut usize,
) -> i32 {
    match resolve_with_context(node, |node, _, context| list(node, context, query)).flatten() {
        None => 0,
        Some(List::Nodes(nodes)) => {
            unsafe { give_list(&nodes, out_ptr, out_len) };
            1
        }
        Some(List::ViewParent) => 2,
    }
}

enum Frame {
    Zero,
    Rect(Rect),
    View,
}

fn frame(node: &Node, query: i32, range: Range) -> Frame {
    match query {
        query::FRAME => node.bounding_box().map_or_else(
            || if node.is_root() { Frame::View } else { Frame::Zero },
            Frame::Rect,
        ),
        query::FRAME_FOR_RANGE => node
            .supports_text_ranges()
            .then(|| from_ns_range(node, range))
            .flatten()
            .and_then(|range| range.bounding_boxes().into_iter().reduce(|rect1, rect2| rect1.union(rect2)))
            .map_or(Frame::Zero, Frame::Rect),
        _ => Frame::Zero,
    }
}

pub unsafe extern "C" fn node_rect(node: *mut c_void, query: i32, location: usize, length: usize, out: *mut f64) -> i32 {
    match resolve(node, |node| frame(node, query, Range { location, length })) {
        Some(Frame::Rect(rect)) => {
            unsafe {
                out.write(rect.x0);
                out.add(1).write(rect.y0);
                out.add(2).write(rect.x1);
                out.add(3).write(rect.y1);
            }
            1
        }
        Some(Frame::View) => 2,
        Some(Frame::Zero) | None => 0,
    }
}

fn text_range(node: &Node, query: i32, argument: isize, point: Point) -> Option<Range> {
    if !node.supports_text_ranges() {
        return None;
    }
    let position = || usize::try_from(argument).ok().and_then(|index| node.text_position_from_global_utf16_index(index));
    match query {
        query::SELECTED_TEXT_RANGE => node.text_selection().map(|range| to_ns_range(&range)),
        query::RANGE_FOR_LINE => usize::try_from(argument)
            .ok()
            .and_then(|line| node.line_range_from_index(line))
            .map(|range| to_ns_range(&range)),
        query::RANGE_FOR_POSITION => {
            let point = node.transform().inverse() * point;
            Some(to_ns_range_for_character(&node.text_position_at_point(point)))
        }
        query::RANGE_FOR_INDEX => position().map(|pos| to_ns_range_for_character(&pos)),
        query::STYLE_RANGE_FOR_INDEX => position().map(|pos| {
            let start = if pos.is_format_start() { pos } else { pos.backward_to_format_start() };
            let mut range = start.to_degenerate_range();
            range.set_end(pos.forward_to_format_end());
            to_ns_range(&range)
        }),
        _ => None,
    }
}

pub unsafe extern "C" fn node_range(
    node: *mut c_void,
    query: i32,
    argument: isize,
    x: f64,
    y: f64,
    out_location: *mut usize,
    out_length: *mut usize,
) -> bool {
    let range = resolve(node, |node| text_range(node, query, argument, Point::new(x, y))).flatten();
    range.is_some_and(|range| {
        unsafe {
            out_location.write(range.location);
            out_length.write(range.length);
        }
        true
    })
}

fn packed_color(color: Option<Color>) -> u64 {
    color.map_or(u64::MAX, |color| {
        (u64::from(color.red) << 24) | (u64::from(color.green) << 16) | (u64::from(color.blue) << 8) | u64::from(color.alpha)
    })
}

fn text_alignment(node: &Node) -> i32 {
    match node.text_align() {
        Some(TextAlign::Left) => 0,
        Some(TextAlign::Center) => 1,
        Some(TextAlign::Right) => 2,
        Some(TextAlign::Justify) => 3,
        None => -1,
    }
}

fn emit_text_run(node: &Node, text: &str, sink: *mut c_void, run: ffi::TextRun) {
    let family = node.font_family();
    let language = node.language();
    unsafe {
        run(
            sink,
            text.as_ptr(),
            text.len(),
            packed_color(node.background_color()),
            packed_color(node.foreground_color()),
            family.map_or(ptr::null(), str::as_ptr),
            family.map_or(0, str::len),
            node.font_size().map_or(f64::NAN, f64::from),
            node.font_weight().is_some_and(|weight| weight >= 700.0),
            node.is_italic(),
            packed_color(node.underline().map(|decoration| decoration.color)),
            packed_color(node.strikethrough().map(|decoration| decoration.color)),
            language.map_or(ptr::null(), str::as_ptr),
            language.map_or(0, str::len),
            text_alignment(node),
        );
    }
}

pub unsafe extern "C" fn node_text_runs(
    node: *mut c_void,
    location: usize,
    length: usize,
    sink: *mut c_void,
    run: ffi::TextRun,
) -> bool {
    resolve(node, |node| {
        let Some(range) = node
            .supports_text_ranges()
            .then(|| from_ns_range(node, Range { location, length }))
            .flatten()
        else {
            return false;
        };
        range.traverse_text::<_, ()>(|node, text| {
            emit_text_run(node, text, sink, run);
            None
        });
        true
    })
    .unwrap_or(false)
}

fn request(tree: &Tree, context: &Context, id: NodeId, action: Action, data: Option<ActionData>) {
    if let Some((target_node, target_tree)) = tree.state().locate_node(id) {
        context.do_action(ActionRequest {
            action,
            target_tree,
            target_node,
            data,
        });
    }
}

fn set_focused(node: &Node, tree: &Tree, context: &Context, focused: bool) {
    if focused {
        if node.is_focusable(&filter) {
            request(tree, context, node.id(), Action::Focus, None);
        }
    } else {
        let root = tree.state().root();
        if root.is_focusable(&filter) {
            request(tree, context, root.id(), Action::Focus, None);
        }
    }
}

fn perform(node: &Node, tree: &Tree, context: &Context, action: i32, number: f64, range: Range, text: Option<String>) -> bool {
    let id = node.id();
    let wrapper = NodeWrapper(node);
    let selectable = || node.is_clickable(&filter) && wrapper.is_item_like() && node.is_selectable();
    let request_if = |allowed: bool, action: Action| {
        if allowed {
            request(tree, context, id, action, None);
        }
        allowed
    };
    match action {
        query::SET_VALUE_STRING => {
            if let Some(text) = text {
                request(tree, context, id, Action::SetValue, Some(ActionData::Value(text.into())));
            }
            false
        }
        query::SET_VALUE_NUMBER => {
            request(tree, context, id, Action::SetValue, Some(ActionData::NumericValue(number)));
            false
        }
        query::SET_FOCUSED => {
            set_focused(node, tree, context, number > 0.0);
            false
        }
        query::PRESS => request_if(node.is_clickable(&filter), Action::Click),
        query::INCREMENT => request_if(node.supports_increment(&filter), Action::Increment),
        query::DECREMENT => request_if(node.supports_decrement(&filter), Action::Decrement),
        query::SET_SELECTED_TEXT_RANGE => {
            if let Some(range) = node.supports_text_ranges().then(|| from_ns_range(node, range)).flatten() {
                request(
                    tree,
                    context,
                    id,
                    Action::SetTextSelection,
                    Some(ActionData::SetTextSelection(range.to_text_selection())),
                );
            }
            false
        }
        query::SET_SELECTED => {
            if selectable() && node.is_selected() != Some(number > 0.0) {
                request(tree, context, id, Action::Click, None);
            }
            false
        }
        query::PICK => request_if(selectable(), Action::Click),
        query::SCROLL_TO_VISIBLE => {
            request(tree, context, id, Action::ScrollIntoView, None);
            false
        }
        _ => false,
    }
}

pub unsafe extern "C" fn node_action(
    node: *mut c_void,
    action: i32,
    number: f64,
    location: usize,
    length: usize,
    text: *const u8,
    text_len: usize,
) -> bool {
    let text = (!text.is_null())
        .then(|| String::from_utf8_lossy(unsafe { core::slice::from_raw_parts(text, text_len) }).into_owned());
    resolve_with_context(node, |node, tree, context| {
        perform(node, tree, context, action, number, Range { location, length }, text)
    })
    .unwrap_or(false)
}

pub unsafe extern "C" fn node_release(node: *mut c_void) {
    drop(unsafe { Box::from_raw(node.cast::<NodeRef>()) });
}
