# COVERAGE

- Scope: an AccessKit adapter for `AppKit`. It covers the functions of `accesskit_macos` 0.26.
- Legend: `✅` implemented, `⏭️` not implemented

## Adapters

| AccessKit API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| subclassing adapter for a view | ✅ | `SubclassingAdapter::new` | Changes the class of the view at run time. |
| subclassing adapter for a window | ✅ | `SubclassingAdapter::for_window` | Uses the content view of the window. |
| adapter for a custom view | ✅ | `Adapter::new`, `Adapter::view_children`, `Adapter::focus`, `Adapter::hit_test` | The functions give autoreleased objects. |
| tree updates | ✅ | `update_if_active` |  |
| host focus | ✅ | `update_view_focus_state` |  |
| events | ✅ | `QueuedEvents::raise` |  |
| window focus forwarder | ✅ | `add_focus_forwarder_to_window_class` |  |

## NSView methods of the subclass

| Apple API | Status | Notes |
| --- | --- | --- |
| `accessibilityChildren` | ✅ | The root node, or its children when the root is not included. |
| `accessibilityFocusedUIElement` | ✅ |  |
| `accessibilityHitTest:` | ✅ |  |
| `superclass` | ✅ | Gives the superclass of the original class. |

## NSAccessibilityElement methods of a node

| Apple API | Status | Notes |
| --- | --- | --- |
| `accessibilityParent`, `accessibilityWindow`, `accessibilityTopLevelUIElement` | ✅ |  |
| `accessibilityChildren`, `accessibilityChildrenInNavigationOrder`, `accessibilitySelectedChildren` | ✅ |  |
| `accessibilityFrame` | ✅ | Physical pixels to screen points. |
| `accessibilityRole`, `accessibilitySubrole`, `accessibilityRoleDescription` | ✅ |  |
| `accessibilityIdentifier`, `accessibilityTitle`, `accessibilityHelp`, `accessibilityPlaceholderValue` | ✅ |  |
| `accessibilityValue`, `setAccessibilityValue:`, `accessibilityMinValue`, `accessibilityMaxValue` | ✅ |  |
| `accessibilityURL`, `accessibilityOrientation` | ✅ |  |
| `isAccessibilityElement`, `isAccessibilityFocused`, `setAccessibilityFocused:`, `isAccessibilityEnabled` | ✅ |  |
| `isAccessibilityRequired`, `isAccessibilitySelected`, `setAccessibilitySelected:`, `isAccessibilityModal` | ✅ |  |
| `accessibilityPerformPress`, `accessibilityPerformIncrement`, `accessibilityPerformDecrement`, `accessibilityPerformPick` | ✅ |  |
| `accessibilityNumberOfCharacters`, `accessibilitySelectedText`, `accessibilitySelectedTextRange`, `setAccessibilitySelectedTextRange:` | ✅ |  |
| `accessibilityInsertionPointLineNumber`, `accessibilityRangeForLine:`, `accessibilityLineForIndex:` | ✅ |  |
| `accessibilityRangeForPosition:`, `accessibilityRangeForIndex:`, `accessibilityStyleRangeForIndex:` | ✅ |  |
| `accessibilityStringForRange:`, `accessibilityAttributedStringForRange:`, `accessibilityFrameForRange:` | ✅ | The attributed string has colors, fonts, decorations, the language and the alignment. |
| `accessibilityRows`, `accessibilitySelectedRows`, `accessibilityTabs`, `accessibilityLinkedUIElements` | ✅ |  |
| `accessibilityActionNames`, `accessibilityPerformAction:` | ✅ | `AXScrollToVisible`. |
| `accessibilityAttributeValue:` | ✅ | `AXBrailleLabel` and `AXBrailleRoleDescription`. |
| `isAccessibilitySelectorAllowed:` | ✅ |  |
| `accessibilityNotifiesWhenDestroyed` | ✅ |  |
| `accessibilityCustomActions`, `accessibilityCustomRotors` | ⏭️ |  |

## Notifications

| Apple API | Status |
| --- | --- |
| `NSAccessibilityFocusedUIElementChangedNotification` | ✅ |
| `NSAccessibilityValueChangedNotification` | ✅ |
| `NSAccessibilitySelectedTextChangedNotification` | ✅ |
| `NSAccessibilityTitleChangedNotification` | ✅ |
| `NSAccessibilitySelectedRowsChangedNotification` | ✅ |
| `NSAccessibilityUIElementDestroyedNotification` | ✅ |
| `NSAccessibilityAnnouncementRequestedNotification` | ✅ |
