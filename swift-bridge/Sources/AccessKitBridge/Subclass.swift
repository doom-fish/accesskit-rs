// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

import AppKit
import ObjectiveC

final class AccessKitAttachment: NSObject {
    var state: UnsafeMutableRawPointer?
    let previousClass: AnyClass

    init(state: UnsafeMutableRawPointer, previousClass: AnyClass) {
        self.state = state
        self.previousClass = previousClass
        super.init()
    }
}

private let attachmentKey = UnsafeMutableRawPointer.allocate(byteCount: 1, alignment: 1)

private var subclasses: [ObjectIdentifier: AnyClass] = [:]

private func attachment(of view: AnyObject) -> AccessKitAttachment? {
    objc_getAssociatedObject(view, attachmentKey) as? AccessKitAttachment
}

private func implementation(_ block: Any) -> IMP {
    imp_implementationWithBlock(block)
}

private func subclass(of previous: AnyClass) -> AnyClass {
    if let existing = subclasses[ObjectIdentifier(previous)] {
        return existing
    }
    let name = "AccessKitRsSubclassOf" + NSStringFromClass(previous)
    guard let created = objc_allocateClassPair(previous, name, 0) else {
        fatalError("accesskit-rs could not make the class \(name)")
    }
    let superclass: @convention(block) (AnyObject) -> AnyClass? = { view in
        attachment(of: view).flatMap { class_getSuperclass($0.previousClass) }
    }
    let children: @convention(block) (NSView) -> NSArray? = { view in
        guard let state = attachment(of: view)?.state else {
            return nil
        }
        var pointer: UnsafePointer<UnsafeMutableRawPointer?>?
        var count: UInt = 0
        accessKitCallbacks.viewChildren(state, &pointer, &count)
        return accessKitNodes(pointer, count) as NSArray
    }
    let focus: @convention(block) (NSView) -> AnyObject? = { view in
        guard let state = attachment(of: view)?.state, let node = accessKitCallbacks.viewFocus(state) else {
            return nil
        }
        return Unmanaged<AnyObject>.fromOpaque(node).takeUnretainedValue()
    }
    let hitTest: @convention(block) (NSView, NSPoint) -> AnyObject? = { view, point in
        guard let state = attachment(of: view)?.state, let window = view.window else {
            return nil
        }
        let pixel = accessKitPixelPoint(view: view, window: window, screenPoint: point)
        guard let node = accessKitCallbacks.viewHitTest(state, pixel.x, pixel.y) else {
            return nil
        }
        return Unmanaged<AnyObject>.fromOpaque(node).takeUnretainedValue()
    }
    class_addMethod(created, NSSelectorFromString("superclass"), implementation(superclass), "#@:")
    class_addMethod(created, NSSelectorFromString("accessibilityChildren"), implementation(children), "@@:")
    class_addMethod(created, NSSelectorFromString("accessibilityFocusedUIElement"), implementation(focus), "@@:")
    class_addMethod(created, NSSelectorFromString("accessibilityHitTest:"), implementation(hitTest), "@@:{CGPoint=dd}")
    objc_registerClassPair(created)
    subclasses[ObjectIdentifier(previous)] = created
    return created
}

@_cdecl("accesskit_rs_window_content_view")
public func accesskit_rs_window_content_view(_ window: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let window: NSWindow = accessKitObject(window)
    return window.contentView.map { Unmanaged.passRetained($0).toOpaque() }
}

@_cdecl("accesskit_rs_subclass_attach")
public func accesskit_rs_subclass_attach(
    _ view: UnsafeMutableRawPointer,
    _ state: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    let view: NSView = accessKitObject(view)
    guard attachment(of: view) == nil, let previous = object_getClass(view) else {
        return nil
    }
    let attached = AccessKitAttachment(state: state, previousClass: previous)
    objc_setAssociatedObject(view, attachmentKey, attached, .OBJC_ASSOCIATION_RETAIN_NONATOMIC)
    object_setClass(view, subclass(of: previous))
    return Unmanaged.passRetained(attached).toOpaque()
}

@_cdecl("accesskit_rs_subclass_detach")
public func accesskit_rs_subclass_detach(
    _ view: UnsafeMutableRawPointer,
    _ attachment: UnsafeMutableRawPointer
) {
    let view: NSView = accessKitObject(view)
    let attached: AccessKitAttachment = accessKitObject(attachment)
    attached.state = nil
    object_setClass(view, attached.previousClass)
    objc_setAssociatedObject(view, attachmentKey, nil, .OBJC_ASSOCIATION_RETAIN_NONATOMIC)
}

@_cdecl("accesskit_rs_add_focus_forwarder")
public func accesskit_rs_add_focus_forwarder(_ className: UnsafePointer<CChar>) -> Int32 {
    guard let windowClass = NSClassFromString(String(cString: className)) else {
        return 1
    }
    let forward: @convention(block) (NSWindow) -> AnyObject? = { window in
        window.contentView?.accessibilityFocusedUIElement as AnyObject?
    }
    let added = class_addMethod(
        windowClass,
        NSSelectorFromString("accessibilityFocusedUIElement"),
        implementation(forward),
        "@@:"
    )
    return added ? 0 : 2
}
