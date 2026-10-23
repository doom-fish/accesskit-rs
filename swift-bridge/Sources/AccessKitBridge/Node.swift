// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

import AppKit

enum AccessKitStringQuery: Int32 {
    case role = 0
    case subrole
    case roleDescription
    case identifier
    case title
    case help
    case placeholder
    case url
    case selectedText
    case stringForRange
    case brailleLabel
    case brailleRoleDescription
}

enum AccessKitNumberQuery: Int32 {
    case minValue = 0
    case maxValue
}

enum AccessKitIntegerQuery: Int32 {
    case orientation = 0
    case numberOfCharacters
    case insertionPointLineNumber
    case lineForIndex
}

enum AccessKitFlagQuery: Int32 {
    case isElement = 0
    case isFocused
    case isEnabled
    case isRequired
    case isSelected
    case isModal
    case selectorAllowed
    case supportsScrollToVisible
}

enum AccessKitListQuery: Int32 {
    case children = 0
    case childrenInNavigationOrder
    case selectedChildren
    case rows
    case selectedRows
    case linked
    case tabs
    case parent
    case window
    case topLevel
}

enum AccessKitRectQuery: Int32 {
    case frame = 0
    case frameForRange
}

enum AccessKitRangeQuery: Int32 {
    case selectedTextRange = 0
    case rangeForLine
    case rangeForPosition
    case rangeForIndex
    case styleRangeForIndex
}

enum AccessKitAction: Int32 {
    case setValueString = 0
    case setValueNumber
    case setFocused
    case press
    case increment
    case decrement
    case setSelectedTextRange
    case setSelected
    case pick
    case scrollToVisible
}

private let scrollToVisibleAction = NSAccessibility.Action(rawValue: "AXScrollToVisible")

private let selectorGroups: [String: Int32] = {
    var groups: [String: Int32] = [
        "setAccessibilityFocused:": 1,
        "accessibilityPerformPress": 2,
        "accessibilityPerformIncrement": 3,
        "accessibilityPerformDecrement": 4,
        "setAccessibilityValue:": 6,
        "isAccessibilitySelected": 7,
        "accessibilityRows": 8,
        "accessibilitySelectedRows": 8,
        "setAccessibilitySelected:": 9,
        "accessibilityPerformPick": 9,
        "accessibilityTabs": 10,
        "isAccessibilityModal": 11,
        "accessibilityAttributeValue:": 12,
        "accessibilityURL": 13,
    ]
    for selector in [
        "accessibilityNumberOfCharacters",
        "accessibilitySelectedText",
        "accessibilitySelectedTextRange",
        "accessibilityInsertionPointLineNumber",
        "accessibilityRangeForLine:",
        "accessibilityRangeForPosition:",
        "accessibilityStringForRange:",
        "accessibilityAttributedStringForRange:",
        "accessibilityFrameForRange:",
        "accessibilityLineForIndex:",
        "accessibilityRangeForIndex:",
        "accessibilityStyleRangeForIndex:",
        "setAccessibilitySelectedTextRange:",
    ] {
        groups[selector] = 5
    }
    for selector in [
        "accessibilityParent",
        "accessibilityChildren",
        "accessibilityChildrenInNavigationOrder",
        "accessibilitySelectedChildren",
        "accessibilityFrame",
        "accessibilityRole",
        "accessibilitySubrole",
        "isAccessibilityEnabled",
        "accessibilityWindow",
        "accessibilityTopLevelUIElement",
        "accessibilityLinkedUIElements",
        "accessibilityRoleDescription",
        "accessibilityIdentifier",
        "accessibilityTitle",
        "accessibilityHelp",
        "accessibilityPlaceholderValue",
        "accessibilityValue",
        "accessibilityMinValue",
        "accessibilityMaxValue",
        "isAccessibilityRequired",
        "accessibilityOrientation",
        "isAccessibilityElement",
        "isAccessibilityFocused",
        "accessibilityNotifiesWhenDestroyed",
        "isAccessibilitySelectorAllowed:",
        "accessibilityActionNames",
        "accessibilityPerformAction:",
    ] {
        groups[selector] = 0
    }
    return groups
}()

private let textAlignments: [NSTextAlignment] = [.left, .center, .right, .justified]

private func accessKitColor(_ packed: UInt64) -> CGColor? {
    guard packed != UInt64.max else {
        return nil
    }
    let channel = { (shift: UInt64) in CGFloat((packed >> shift) & 0xFF) / 255.0 }
    return NSColor(srgbRed: channel(24), green: channel(16), blue: channel(8), alpha: channel(0)).cgColor
}

private let appendTextRun: AccessKitTextRunCallback = {
    sink, text, textLength, background, foreground, family, familyLength, size, bold, italic,
    underline, strikethrough, language, languageLength, alignment in
    guard let sink else {
        return
    }
    let result = Unmanaged<NSMutableAttributedString>.fromOpaque(sink).takeUnretainedValue()
    var attributes: [NSAttributedString.Key: Any] = [:]
    if let color = accessKitColor(background) {
        attributes[.accessibilityBackgroundColor] = color
    }
    if let color = accessKitColor(foreground) {
        attributes[.accessibilityForegroundColor] = color
    }
    var font: [NSAccessibility.FontAttributeKey: Any] = [:]
    if let family = accessKitString(family, familyLength) {
        font[.fontFamily] = family
    }
    if !size.isNaN {
        font[.fontSize] = NSNumber(value: Float(size))
    }
    if bold {
        font[NSAccessibility.FontAttributeKey(rawValue: "AXFontBold")] = NSNumber(value: true)
    }
    if italic {
        font[NSAccessibility.FontAttributeKey(rawValue: "AXFontItalic")] = NSNumber(value: true)
    }
    if !font.isEmpty {
        attributes[.accessibilityFont] = font
    }
    if let color = accessKitColor(underline) {
        attributes[.accessibilityUnderline] = NSNumber(value: true)
        attributes[.accessibilityUnderlineColor] = color
    }
    if let color = accessKitColor(strikethrough) {
        attributes[.accessibilityStrikethrough] = NSNumber(value: true)
        attributes[.accessibilityStrikethroughColor] = color
    }
    if let language = accessKitString(language, languageLength) {
        attributes[.accessibilityLanguage] = language
    }
    if alignment >= 0, Int(alignment) < textAlignments.count {
        attributes[.accessibilityAlignment] = NSNumber(value: textAlignments[Int(alignment)].rawValue)
    }
    let string = accessKitString(text, textLength) ?? ""
    result.append(NSAttributedString(string: string, attributes: attributes))
}

final class AccessKitNode: NSAccessibilityElement, NSAccessibilityElementProtocol {
    let reference: AccessKitViewRef
    let node: UnsafeMutableRawPointer

    init(reference: AccessKitViewRef, node: UnsafeMutableRawPointer) {
        self.reference = reference
        self.node = node
        super.init()
    }

    deinit {
        accessKitCallbacks.release(node)
    }

    private func string(_ query: AccessKitStringQuery, _ range: NSRange? = nil) -> (Int32, String?) {
        var pointer: UnsafePointer<UInt8>?
        var length: UInt = 0
        let status = accessKitCallbacks.string(
            node,
            query.rawValue,
            UInt(bitPattern: range?.location ?? 0),
            UInt(bitPattern: range?.length ?? 0),
            &pointer,
            &length
        )
        guard status == 1 else {
            return (status, nil)
        }
        return (status, accessKitString(pointer, length))
    }

    private func number(_ query: AccessKitNumberQuery) -> NSNumber? {
        var value = 0.0
        return accessKitCallbacks.number(node, query.rawValue, &value) ? NSNumber(value: value) : nil
    }

    private func integer(_ query: AccessKitIntegerQuery, _ argument: Int = 0) -> Int {
        var value = 0
        return accessKitCallbacks.integer(node, query.rawValue, argument, &value) ? value : 0
    }

    private func flag(_ query: AccessKitFlagQuery, _ argument: Int32 = 0) -> Bool {
        accessKitCallbacks.flag(node, query.rawValue, argument)
    }

    private func nodes(_ query: AccessKitListQuery) -> (Int32, [AccessKitNode]) {
        var pointer: UnsafePointer<UnsafeMutableRawPointer?>?
        var count: UInt = 0
        let status = accessKitCallbacks.list(node, query.rawValue, &pointer, &count)
        return (status, status == 1 ? accessKitNodes(pointer, count) : [])
    }

    private func list(_ query: AccessKitListQuery) -> [Any]? {
        let (status, nodes) = nodes(query)
        return status == 1 ? nodes : nil
    }

    private func related(_ query: AccessKitListQuery) -> Any? {
        let (status, nodes) = nodes(query)
        switch status {
        case 1:
            return nodes.first
        case 2:
            return reference.view?.accessibilityParent()
        default:
            return nil
        }
    }

    private func range(_ query: AccessKitRangeQuery, _ argument: Int = 0, _ point: NSPoint = .zero) -> NSRange {
        var location: UInt = 0
        var length: UInt = 0
        guard accessKitCallbacks.range(node, query.rawValue, argument, point.x, point.y, &location, &length) else {
            return NSRange(location: 0, length: 0)
        }
        return NSRange(location: Int(bitPattern: location), length: Int(bitPattern: length))
    }

    private func rect(_ query: AccessKitRectQuery, _ range: NSRange? = nil) -> (Int32, [Double]) {
        var rect = [Double](repeating: 0, count: 4)
        let status = accessKitCallbacks.rect(
            node,
            query.rawValue,
            UInt(bitPattern: range?.location ?? 0),
            UInt(bitPattern: range?.length ?? 0),
            &rect
        )
        return (status, rect)
    }

    @discardableResult
    private func perform(
        _ action: AccessKitAction,
        number: Double = 0,
        range: NSRange? = nil,
        text: String? = nil
    ) -> Bool {
        let location = UInt(bitPattern: range?.location ?? 0)
        let length = UInt(bitPattern: range?.length ?? 0)
        guard var text else {
            return accessKitCallbacks.action(node, action.rawValue, number, location, length, nil, 0)
        }
        return text.withUTF8 { buffer in
            accessKitCallbacks.action(
                node,
                action.rawValue,
                number,
                location,
                length,
                buffer.baseAddress,
                UInt(buffer.count)
            )
        }
    }

    override func accessibilityParent() -> Any? {
        related(.parent)
    }

    override func accessibilityWindow() -> Any? {
        related(.window)
    }

    override func accessibilityTopLevelUIElement() -> Any? {
        related(.topLevel)
    }

    override func accessibilityChildren() -> [Any]? {
        list(.children)
    }

    override func accessibilityChildrenInNavigationOrder() -> [NSAccessibilityElementProtocol]? {
        let (status, nodes) = nodes(.childrenInNavigationOrder)
        return status == 1 ? nodes.map { $0 as NSAccessibilityElementProtocol } : nil
    }

    override func accessibilitySelectedChildren() -> [Any]? {
        list(.selectedChildren)
    }

    override func accessibilityFrame() -> NSRect {
        guard let view = reference.view else {
            return .zero
        }
        let (status, frame) = rect(.frame)
        switch status {
        case 1:
            return reference.screenRect(frame)
        case 2:
            return view.accessibilityFrame()
        default:
            return .zero
        }
    }

    override func accessibilityRole() -> NSAccessibility.Role? {
        NSAccessibility.Role(rawValue: string(.role).1 ?? NSAccessibility.Role.unknown.rawValue)
    }

    override func accessibilitySubrole() -> NSAccessibility.Subrole? {
        NSAccessibility.Subrole(rawValue: string(.subrole).1 ?? NSAccessibility.Subrole.unknown.rawValue)
    }

    override func accessibilityRoleDescription() -> String? {
        let (status, description) = string(.roleDescription)
        return status == 2 ? super.accessibilityRoleDescription() : description
    }

    override func accessibilityIdentifier() -> String {
        string(.identifier).1 ?? ""
    }

    override func accessibilityTitle() -> String? {
        string(.title).1
    }

    override func accessibilityHelp() -> String? {
        string(.help).1
    }

    override func accessibilityPlaceholderValue() -> String? {
        string(.placeholder).1
    }

    override func accessibilityValue() -> Any? {
        var number = 0.0
        var pointer: UnsafePointer<UInt8>?
        var length: UInt = 0
        switch accessKitCallbacks.value(node, &number, &pointer, &length) {
        case 1:
            return NSNumber(value: number > 0)
        case 2:
            return NSNumber(value: number)
        case 3:
            return accessKitString(pointer, length)
        default:
            return nil
        }
    }

    override func setAccessibilityValue(_ value: Any?) {
        if let text = value as? NSString {
            perform(.setValueString, text: text as String)
        } else if let number = value as? NSNumber {
            perform(.setValueNumber, number: number.doubleValue)
        }
    }

    override func accessibilityMinValue() -> Any? {
        number(.minValue)
    }

    override func accessibilityMaxValue() -> Any? {
        number(.maxValue)
    }

    override func accessibilityURL() -> URL? {
        string(.url).1.flatMap { NSURL(string: $0) as URL? }
    }

    override func accessibilityOrientation() -> NSAccessibilityOrientation {
        switch integer(.orientation) {
        case 1:
            return .horizontal
        case 2:
            return .vertical
        default:
            return .unknown
        }
    }

    override func isAccessibilityElement() -> Bool {
        flag(.isElement)
    }

    override func isAccessibilityFocused() -> Bool {
        flag(.isFocused)
    }

    override func isAccessibilityEnabled() -> Bool {
        flag(.isEnabled)
    }

    override func setAccessibilityFocused(_ focused: Bool) {
        perform(.setFocused, number: focused ? 1 : 0)
    }

    override func accessibilityPerformPress() -> Bool {
        perform(.press)
    }

    override func accessibilityPerformIncrement() -> Bool {
        perform(.increment)
    }

    override func accessibilityPerformDecrement() -> Bool {
        perform(.decrement)
    }

    override var accessibilityNotifiesWhenDestroyed: Bool {
        true
    }

    override func accessibilityNumberOfCharacters() -> Int {
        integer(.numberOfCharacters)
    }

    override func accessibilitySelectedText() -> String? {
        string(.selectedText).1
    }

    override func accessibilitySelectedTextRange() -> NSRange {
        range(.selectedTextRange)
    }

    override func accessibilityInsertionPointLineNumber() -> Int {
        integer(.insertionPointLineNumber)
    }

    override func accessibilityRange(forLine line: Int) -> NSRange {
        range(.rangeForLine, line)
    }

    override func accessibilityRange(for point: NSPoint) -> NSRange {
        guard let pixel = reference.pixelPoint(screen: point) else {
            return NSRange(location: 0, length: 0)
        }
        return range(.rangeForPosition, 0, pixel)
    }

    override func accessibilityString(for range: NSRange) -> String? {
        string(.stringForRange, range).1
    }

    override func accessibilityAttributedString(for range: NSRange) -> NSAttributedString? {
        let result = NSMutableAttributedString()
        result.beginEditing()
        let found = accessKitCallbacks.textRuns(
            node,
            UInt(bitPattern: range.location),
            UInt(bitPattern: range.length),
            Unmanaged.passUnretained(result).toOpaque(),
            appendTextRun
        )
        result.endEditing()
        return found ? result : nil
    }

    override func accessibilityFrame(for range: NSRange) -> NSRect {
        guard reference.view != nil else {
            return .zero
        }
        let (status, frame) = rect(.frameForRange, range)
        return status == 1 ? reference.screenRect(frame) : .zero
    }

    override func accessibilityLine(for index: Int) -> Int {
        integer(.lineForIndex, index)
    }

    override func accessibilityRange(for index: Int) -> NSRange {
        range(.rangeForIndex, index)
    }

    override func accessibilityStyleRange(for index: Int) -> NSRange {
        range(.styleRangeForIndex, index)
    }

    override func setAccessibilitySelectedTextRange(_ range: NSRange) {
        perform(.setSelectedTextRange, range: range)
    }

    override func isAccessibilityRequired() -> Bool {
        flag(.isRequired)
    }

    override func isAccessibilitySelected() -> Bool {
        flag(.isSelected)
    }

    override func setAccessibilitySelected(_ selected: Bool) {
        perform(.setSelected, number: selected ? 1 : 0)
    }

    override func accessibilityAttributeValue(_ attribute: NSAccessibility.Attribute) -> Any? {
        switch attribute.rawValue {
        case "AXBrailleLabel":
            return string(.brailleLabel).1
        case "AXBrailleRoleDescription":
            return string(.brailleRoleDescription).1
        default:
            return nil
        }
    }

    override func accessibilityRows() -> [Any]? {
        list(.rows)
    }

    override func accessibilitySelectedRows() -> [Any]? {
        list(.selectedRows)
    }

    override func accessibilityPerformPick() -> Bool {
        perform(.pick)
    }

    override func accessibilityLinkedUIElements() -> [Any]? {
        list(.linked)
    }

    override func accessibilityTabs() -> [Any]? {
        list(.tabs)
    }

    override func isAccessibilityModal() -> Bool {
        flag(.isModal)
    }

    override func accessibilityActionNames() -> [NSAccessibility.Action] {
        flag(.supportsScrollToVisible) ? [scrollToVisibleAction] : []
    }

    override func accessibilityPerformAction(_ action: NSAccessibility.Action) {
        if action == scrollToVisibleAction {
            perform(.scrollToVisible)
        }
    }

    override func isAccessibilitySelectorAllowed(_ selector: Selector) -> Bool {
        guard let group = selectorGroups[NSStringFromSelector(selector)] else {
            return false
        }
        return flag(.selectorAllowed, group)
    }
}
