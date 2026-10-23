use core::ffi::{c_char, c_void};

pub type NodeString = unsafe extern "C" fn(*mut c_void, i32, usize, usize, *mut *const u8, *mut usize) -> i32;
pub type NodeNumber = unsafe extern "C" fn(*mut c_void, i32, *mut f64) -> bool;
pub type NodeInteger = unsafe extern "C" fn(*mut c_void, i32, isize, *mut isize) -> bool;
pub type NodeFlag = unsafe extern "C" fn(*mut c_void, i32, i32) -> bool;
pub type NodeValue = unsafe extern "C" fn(*mut c_void, *mut f64, *mut *const u8, *mut usize) -> i32;
pub type NodeList = unsafe extern "C" fn(*mut c_void, i32, *mut *const *mut c_void, *mut usize) -> i32;
pub type NodeRect = unsafe extern "C" fn(*mut c_void, i32, usize, usize, *mut f64) -> i32;
pub type NodeRange = unsafe extern "C" fn(*mut c_void, i32, isize, f64, f64, *mut usize, *mut usize) -> bool;
pub type TextRun = unsafe extern "C" fn(
    *mut c_void,
    *const u8,
    usize,
    u64,
    u64,
    *const u8,
    usize,
    f64,
    bool,
    bool,
    u64,
    u64,
    *const u8,
    usize,
    i32,
);
pub type NodeTextRuns = unsafe extern "C" fn(*mut c_void, usize, usize, *mut c_void, TextRun) -> bool;
pub type NodeAction = unsafe extern "C" fn(*mut c_void, i32, f64, usize, usize, *const u8, usize) -> bool;
pub type NodeRelease = unsafe extern "C" fn(*mut c_void);
pub type ViewChildren = unsafe extern "C" fn(*mut c_void, *mut *const *mut c_void, *mut usize);
pub type ViewFocus = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
pub type ViewHitTest = unsafe extern "C" fn(*mut c_void, f64, f64) -> *mut c_void;

extern "C" {
    pub fn accesskit_rs_install(
        string: NodeString,
        number: NodeNumber,
        integer: NodeInteger,
        flag: NodeFlag,
        value: NodeValue,
        list: NodeList,
        rect: NodeRect,
        range: NodeRange,
        text_runs: NodeTextRuns,
        action: NodeAction,
        release: NodeRelease,
        view_children: ViewChildren,
        view_focus: ViewFocus,
        view_hit_test: ViewHitTest,
    );
    pub fn accesskit_rs_retain(object: *mut c_void);
    pub fn accesskit_rs_release(object: *mut c_void);
    pub fn accesskit_rs_autorelease(object: *mut c_void) -> *mut c_void;
    pub fn accesskit_rs_array(objects: *const *mut c_void, count: usize) -> *mut c_void;
    pub fn accesskit_rs_view_ref_new(view: *mut c_void) -> *mut c_void;
    pub fn accesskit_rs_view_ref_pixel_point(reference: *mut c_void, x: f64, y: f64, out: *mut f64) -> bool;
    pub fn accesskit_rs_node_new(reference: *mut c_void, node: *mut c_void) -> *mut c_void;
    pub fn accesskit_rs_post(element: *mut c_void, notification: i32);
    pub fn accesskit_rs_announce(reference: *mut c_void, text: *const u8, length: usize, assertive: bool);
    pub fn accesskit_rs_window_content_view(window: *mut c_void) -> *mut c_void;
    pub fn accesskit_rs_subclass_attach(view: *mut c_void, state: *mut c_void) -> *mut c_void;
    pub fn accesskit_rs_subclass_detach(view: *mut c_void, attachment: *mut c_void);
    pub fn accesskit_rs_add_focus_forwarder(class_name: *const c_char) -> i32;
}
