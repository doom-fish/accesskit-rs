// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

import AppKit

public typealias AccessKitNodeStringCallback = @convention(c) (
    UnsafeMutableRawPointer,
    Int32,
    UInt,
    UInt,
    UnsafeMutablePointer<UnsafePointer<UInt8>?>,
    UnsafeMutablePointer<UInt>
) -> Int32

public typealias AccessKitNodeNumberCallback = @convention(c) (
    UnsafeMutableRawPointer,
    Int32,
    UnsafeMutablePointer<Double>
) -> Bool

public typealias AccessKitNodeIntegerCallback = @convention(c) (
    UnsafeMutableRawPointer,
    Int32,
    Int,
    UnsafeMutablePointer<Int>
) -> Bool

public typealias AccessKitNodeFlagCallback = @convention(c) (
    UnsafeMutableRawPointer,
    Int32,
    Int32
) -> Bool

public typealias AccessKitNodeValueCallback = @convention(c) (
    UnsafeMutableRawPointer,
    UnsafeMutablePointer<Double>,
    UnsafeMutablePointer<UnsafePointer<UInt8>?>,
    UnsafeMutablePointer<UInt>
) -> Int32

public typealias AccessKitNodeListCallback = @convention(c) (
    UnsafeMutableRawPointer,
    Int32,
    UnsafeMutablePointer<UnsafePointer<UnsafeMutableRawPointer?>?>,
    UnsafeMutablePointer<UInt>
) -> Int32

public typealias AccessKitNodeRectCallback = @convention(c) (
    UnsafeMutableRawPointer,
    Int32,
    UInt,
    UInt,
    UnsafeMutablePointer<Double>
) -> Int32

public typealias AccessKitNodeRangeCallback = @convention(c) (
    UnsafeMutableRawPointer,
    Int32,
    Int,
    Double,
    Double,
    UnsafeMutablePointer<UInt>,
    UnsafeMutablePointer<UInt>
) -> Bool

public typealias AccessKitTextRunCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafePointer<UInt8>?,
    UInt,
    UInt64,
    UInt64,
    UnsafePointer<UInt8>?,
    UInt,
    Double,
    Bool,
    Bool,
    UInt64,
    UInt64,
    UnsafePointer<UInt8>?,
    UInt,
    Int32
) -> Void

public typealias AccessKitNodeTextRunsCallback = @convention(c) (
    UnsafeMutableRawPointer,
    UInt,
    UInt,
    UnsafeMutableRawPointer?,
    AccessKitTextRunCallback
) -> Bool

public typealias AccessKitNodeActionCallback = @convention(c) (
    UnsafeMutableRawPointer,
    Int32,
    Double,
    UInt,
    UInt,
    UnsafePointer<UInt8>?,
    UInt
) -> Bool

public typealias AccessKitNodeReleaseCallback = @convention(c) (UnsafeMutableRawPointer) -> Void

public typealias AccessKitViewChildrenCallback = @convention(c) (
    UnsafeMutableRawPointer,
    UnsafeMutablePointer<UnsafePointer<UnsafeMutableRawPointer?>?>,
    UnsafeMutablePointer<UInt>
) -> Void

public typealias AccessKitViewFocusCallback = @convention(c) (UnsafeMutableRawPointer) -> UnsafeMutableRawPointer?

public typealias AccessKitViewHitTestCallback = @convention(c) (
    UnsafeMutableRawPointer,
    Double,
    Double
) -> UnsafeMutableRawPointer?

struct AccessKitCallbacks {
    let string: AccessKitNodeStringCallback
    let number: AccessKitNodeNumberCallback
    let integer: AccessKitNodeIntegerCallback
    let flag: AccessKitNodeFlagCallback
    let value: AccessKitNodeValueCallback
    let list: AccessKitNodeListCallback
    let rect: AccessKitNodeRectCallback
    let range: AccessKitNodeRangeCallback
    let textRuns: AccessKitNodeTextRunsCallback
    let action: AccessKitNodeActionCallback
    let release: AccessKitNodeReleaseCallback
    let viewChildren: AccessKitViewChildrenCallback
    let viewFocus: AccessKitViewFocusCallback
    let viewHitTest: AccessKitViewHitTestCallback
}

private var installedCallbacks: AccessKitCallbacks?

var accessKitCallbacks: AccessKitCallbacks {
    installedCallbacks!
}

@_cdecl("accesskit_rs_install")
public func accesskit_rs_install(
    _ string: AccessKitNodeStringCallback,
    _ number: AccessKitNodeNumberCallback,
    _ integer: AccessKitNodeIntegerCallback,
    _ flag: AccessKitNodeFlagCallback,
    _ value: AccessKitNodeValueCallback,
    _ list: AccessKitNodeListCallback,
    _ rect: AccessKitNodeRectCallback,
    _ range: AccessKitNodeRangeCallback,
    _ textRuns: AccessKitNodeTextRunsCallback,
    _ action: AccessKitNodeActionCallback,
    _ release: AccessKitNodeReleaseCallback,
    _ viewChildren: AccessKitViewChildrenCallback,
    _ viewFocus: AccessKitViewFocusCallback,
    _ viewHitTest: AccessKitViewHitTestCallback
) {
    installedCallbacks = AccessKitCallbacks(
        string: string,
        number: number,
        integer: integer,
        flag: flag,
        value: value,
        list: list,
        rect: rect,
        range: range,
        textRuns: textRuns,
        action: action,
        release: release,
        viewChildren: viewChildren,
        viewFocus: viewFocus,
        viewHitTest: viewHitTest
    )
}

func accessKitObject<T: AnyObject>(_ pointer: UnsafeMutableRawPointer) -> T {
    Unmanaged<T>.fromOpaque(pointer).takeUnretainedValue()
}

func accessKitString(_ pointer: UnsafePointer<UInt8>?, _ length: UInt) -> String? {
    guard let pointer else {
        return nil
    }
    return String(decoding: UnsafeBufferPointer(start: pointer, count: Int(length)), as: UTF8.self)
}

func accessKitNodes(_ pointer: UnsafePointer<UnsafeMutableRawPointer?>?, _ count: UInt) -> [AccessKitNode] {
    guard let pointer else {
        return []
    }
    return UnsafeBufferPointer(start: pointer, count: Int(count)).compactMap { element in
        element.map { Unmanaged<AccessKitNode>.fromOpaque($0).takeUnretainedValue() }
    }
}

func accessKitPixelPoint(view: NSView, window: NSWindow, screenPoint: NSPoint) -> NSPoint {
    let point = view.convert(window.convertPoint(fromScreen: screenPoint), from: nil)
    let factor = window.backingScaleFactor
    let y = view.isFlipped ? point.y * factor : (view.bounds.size.height - point.y) * factor
    return NSPoint(x: point.x * factor, y: y)
}

final class AccessKitViewRef: NSObject {
    weak var view: NSView?

    init(view: NSView) {
        self.view = view
        super.init()
    }

    func screenRect(_ rect: [Double]) -> NSRect {
        guard let view, let window = view.window else {
            return .zero
        }
        let factor = window.backingScaleFactor
        let y = view.isFlipped ? rect[1] / factor : view.bounds.size.height - rect[3] / factor
        let local = NSRect(
            x: rect[0] / factor,
            y: y,
            width: (rect[2] - rect[0]) / factor,
            height: (rect[3] - rect[1]) / factor
        )
        return window.convertToScreen(view.convert(local, to: nil))
    }

    func pixelPoint(screen point: NSPoint) -> NSPoint? {
        guard let view, let window = view.window else {
            return nil
        }
        return accessKitPixelPoint(view: view, window: window, screenPoint: point)
    }
}

@_cdecl("accesskit_rs_retain")
public func accesskit_rs_retain(_ object: UnsafeMutableRawPointer) {
    _ = Unmanaged<AnyObject>.fromOpaque(object).retain()
}

@_cdecl("accesskit_rs_release")
public func accesskit_rs_release(_ object: UnsafeMutableRawPointer) {
    Unmanaged<AnyObject>.fromOpaque(object).release()
}

@_cdecl("accesskit_rs_autorelease")
public func accesskit_rs_autorelease(_ object: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    Unmanaged<AnyObject>.fromOpaque(object).retain().autorelease().toOpaque()
}

@_cdecl("accesskit_rs_array")
public func accesskit_rs_array(
    _ objects: UnsafePointer<UnsafeMutableRawPointer?>?,
    _ count: UInt
) -> UnsafeMutableRawPointer {
    let array = accessKitNodes(objects, count) as NSArray
    return Unmanaged.passRetained(array).autorelease().toOpaque()
}

@_cdecl("accesskit_rs_view_ref_new")
public func accesskit_rs_view_ref_new(_ view: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    Unmanaged.passRetained(AccessKitViewRef(view: accessKitObject(view))).toOpaque()
}

@_cdecl("accesskit_rs_view_ref_pixel_point")
public func accesskit_rs_view_ref_pixel_point(
    _ reference: UnsafeMutableRawPointer,
    _ x: Double,
    _ y: Double,
    _ out: UnsafeMutablePointer<Double>
) -> Bool {
    let reference: AccessKitViewRef = accessKitObject(reference)
    guard let point = reference.pixelPoint(screen: NSPoint(x: x, y: y)) else {
        return false
    }
    out[0] = point.x
    out[1] = point.y
    return true
}

@_cdecl("accesskit_rs_node_new")
public func accesskit_rs_node_new(
    _ reference: UnsafeMutableRawPointer,
    _ node: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer {
    Unmanaged.passRetained(AccessKitNode(reference: accessKitObject(reference), node: node)).toOpaque()
}

private let accessKitNotifications: [NSAccessibility.Notification] = [
    .focusedUIElementChanged,
    .valueChanged,
    .selectedTextChanged,
    .titleChanged,
    .selectedRowsChanged,
    .uiElementDestroyed,
]

@_cdecl("accesskit_rs_post")
public func accesskit_rs_post(_ element: UnsafeMutableRawPointer, _ notification: Int32) {
    let element: AnyObject = accessKitObject(element)
    NSAccessibility.post(element: element, notification: accessKitNotifications[Int(notification)])
}

@_cdecl("accesskit_rs_announce")
public func accesskit_rs_announce(
    _ reference: UnsafeMutableRawPointer,
    _ text: UnsafePointer<UInt8>?,
    _ length: UInt,
    _ assertive: Bool
) {
    let reference: AccessKitViewRef = accessKitObject(reference)
    guard let window = reference.view?.window, let text = accessKitString(text, length) else {
        return
    }
    let priority: NSAccessibilityPriorityLevel = assertive ? .high : .medium
    NSAccessibility.post(
        element: window,
        notification: .announcementRequested,
        userInfo: [.announcement: text, .priority: priority.rawValue]
    )
}
