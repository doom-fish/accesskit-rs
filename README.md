# accesskit-rs

An [AccessKit](https://accesskit.dev) adapter for `AppKit` on macOS. It gives an AccessKit tree to `VoiceOver` and to the other assistive applications of macOS.

The adapter has the same functions as `accesskit_macos`. A Swift bridge does all the `AppKit` work, thus the crate does not use Objective-C runtime bindings.

The crate requires macOS 11 or later and Rust 1.85 or later.

## Quick start

`SubclassingAdapter::for_window` attaches an adapter to the content view of a window. A window library, for example one that gives a `raw-window-handle`, can supply the `NSWindow` pointer.

```rust,no_run
use core::ffi::c_void;

use accesskit::{ActionHandler, ActionRequest, ActivationHandler, Node, NodeId, Role, Tree, TreeId, TreeUpdate};
use accesskit_rs::SubclassingAdapter;

struct InitialTree;

impl ActivationHandler for InitialTree {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        let mut button = Node::new(Role::Button);
        button.set_label("Start");
        let mut window = Node::new(Role::Window);
        window.set_children(vec![NodeId(1)]);
        Some(TreeUpdate {
            nodes: vec![(NodeId(0), window), (NodeId(1), button)],
            tree: Some(Tree::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(1),
        })
    }
}

struct Actions;

impl ActionHandler for Actions {
    fn do_action(&mut self, request: ActionRequest) {
        println!("{:?} on {:?}", request.action, request.target_node);
    }
}

fn attach(ns_window: *mut c_void) -> SubclassingAdapter {
    unsafe { SubclassingAdapter::for_window(ns_window, InitialTree, Actions) }
}
```

## Updates and events

- `SubclassingAdapter::update_if_active` applies a `TreeUpdate`, but only after an assistive application asked for the tree.
- `SubclassingAdapter::update_view_focus_state` tells the adapter that the window became key or stopped being key.
- These functions give `QueuedEvents`. Call `QueuedEvents::raise` when no lock or borrow of the adapter is active.
- AccessKit coordinates are physical pixels from the top left corner of the view. The adapter converts them to screen points.

## Other views and windows

- `SubclassingAdapter::new` attaches an adapter to a view that is not a content view.
- `Adapter` is for an `NSView` subclass that implements `accessibilityChildren`, `accessibilityFocusedUIElement` and `accessibilityHitTest:` itself. Its functions give autoreleased Objective-C objects.
- `add_focus_forwarder_to_window_class` adds `accessibilityFocusedUIElement` to an `NSWindow` subclass. The window then sends the request to its content view.

## The main thread

The adapters belong to the main thread. `Adapter::new` stops with a panic on other threads. The adapters are not `Send` and not `Sync`. The action handler runs on the main thread.

## Differences from `accesskit_macos`

- `Adapter::hit_test` takes the screen point as two `f64` values.
- `Role::Heading` has the role `AXHeading`, as in Chromium and `WebKit`.

## Example

```bash
cargo run --example 01_window
```

The example opens a window with a button and a label. Use `VoiceOver` to read them, or press the button with `VoiceOver`. The example writes each action to standard output.

## Acknowledgements

The tree logic and the role tables come from `accesskit_macos` by the AccessKit authors, under the MIT or Apache-2.0 license. Parts of the role tables come from Chromium, under the license in `LICENSE.chromium`.

## License

MIT OR Apache-2.0
