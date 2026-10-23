# Changelog

This file records the important changes to `accesskit-rs`.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-10-23

### Added

- `SubclassingAdapter`, which gives an AccessKit tree to `AppKit` through an `NSView` or the content view of an `NSWindow`. It changes the class of the view at run time.
- `Adapter`, for an `NSView` subclass that implements the accessibility functions of the view itself.
- `QueuedEvents`, which posts the accessibility notifications and the live region announcements of a tree update.
- `add_focus_forwarder_to_window_class`, which sends the focus request of a window class to its content view.
- A Swift element for each AccessKit node. It implements the `NSAccessibility` protocol: roles, subroles, labels, values, frames, focus, selection, rows, tabs, links, text ranges, attributed text and actions.
- The `01_window` example.
- The functions are the same as in `accesskit_macos` 0.26, with two differences. `Adapter::hit_test` takes the screen point as two `f64` values. `Role::Heading` has the role `AXHeading`.
