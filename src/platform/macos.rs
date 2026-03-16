//! macOS platform-specific window enumeration
//!
//! Uses Core Graphics CGWindowListCopyWindowInfo to enumerate on-screen windows.
//! Filters to normal windows (layer == 0) with non-empty titles. Extracts window
//! ID, title, PID, app name, and bounds from each window's property dictionary.

use anyhow::Result;
use crate::window::WindowInfo;

/// List all visible windows on macOS
///
/// Uses CGWindowListCopyWindowInfo with kCGWindowListOptionOnScreenOnly.
/// Filters to layer-0 windows (normal app windows, not menus/dock).
/// Sorts by app_name then title and assigns sequential indices.
#[cfg(target_os = "macos")]
pub fn list_windows() -> Result<Vec<WindowInfo>> {
    use crate::error::AppError;
    use objc2_core_foundation::{CFDictionary, CFType};
    use objc2_core_graphics::{
        kCGNullWindowID, kCGWindowLayer, kCGWindowName,
        kCGWindowNumber, kCGWindowOwnerName, kCGWindowOwnerPID,
        CGWindowListCopyWindowInfo, CGWindowListOption,
    };

    let window_list = CGWindowListCopyWindowInfo(
        CGWindowListOption::OptionOnScreenOnly,
        kCGNullWindowID,
    );

    let array = window_list
        .ok_or_else(|| AppError::enumeration_failed("CGWindowListCopyWindowInfo returned null - check Screen Recording permission"))?;

    let mut windows: Vec<WindowInfo> = Vec::new();

    for i in 0..array.len() {
        // SAFETY: index is in bounds; we do not mutate the array during iteration
        let raw_ptr = unsafe { array.as_opaque().value_at_index(i as isize) };
        if raw_ptr.is_null() {
            continue;
        }

        // Cast raw pointer to CFType so we can downcast to CFDictionary
        let cf_type: &CFType = unsafe { &*(raw_ptr as *const CFType) };
        let dict = match cf_type.downcast_ref::<CFDictionary>() {
            Some(d) => d,
            None => continue,
        };

        // Filter: only normal windows (layer == 0)
        // SAFETY: extern statics are valid for the lifetime of the process
        let layer = unsafe { get_number_from_dict(dict, &kCGWindowLayer) };
        if layer != Some(0) {
            continue;
        }

        // Get title — skip windows without a title
        let title = match unsafe { get_string_from_dict(dict, &kCGWindowName) } {
            Some(t) if !t.is_empty() => t,
            _ => continue,
        };

        let window_id = unsafe { get_number_from_dict(dict, &kCGWindowNumber) }
            .unwrap_or(0) as u64;
        let pid = unsafe { get_number_from_dict(dict, &kCGWindowOwnerPID) }
            .unwrap_or(0) as u32;
        let app_name = unsafe { get_string_from_dict(dict, &kCGWindowOwnerName) }
            .unwrap_or_else(|| format!("PID:{}", pid));

        let (x, y, width, height) = get_bounds_from_dict(dict);

        windows.push(WindowInfo::new(
            0, // Index assigned after sorting
            window_id,
            title,
            pid,
            app_name,
            x,
            y,
            width,
            height,
        ));
    }

    windows.sort_by(|a, b| {
        a.app_name
            .cmp(&b.app_name)
            .then_with(|| a.title.cmp(&b.title))
    });

    for (i, window) in windows.iter_mut().enumerate() {
        window.index = i;
    }

    Ok(windows)
}

/// Extract a CFString value from a CFDictionary by a CFString key.
/// Returns None if the key is absent or the value is not a CFString.
#[cfg(target_os = "macos")]
fn get_string_from_dict(dict: &objc2_core_foundation::CFDictionary, key: &objc2_core_foundation::CFString) -> Option<String> {
    use core::ffi::c_void;
    use objc2_core_foundation::{CFString, CFType};

    let key_ptr: *const CFString = key;
    let key_void: *const c_void = key_ptr.cast();

    // SAFETY: dict and key are valid references; returned pointer is valid if non-null
    let value_ptr = unsafe { dict.value(key_void) };
    if value_ptr.is_null() {
        return None;
    }

    let cf_type: &CFType = unsafe { &*(value_ptr as *const CFType) };
    cf_type.downcast_ref::<CFString>().map(|s| s.to_string())
}

/// Extract a CFNumber value (as i64) from a CFDictionary by a CFString key.
/// Returns None if the key is absent or the value is not a CFNumber.
#[cfg(target_os = "macos")]
fn get_number_from_dict(dict: &objc2_core_foundation::CFDictionary, key: &objc2_core_foundation::CFString) -> Option<i64> {
    use core::ffi::c_void;
    use objc2_core_foundation::{CFNumber, CFNumberType, CFString, CFType};

    let key_ptr: *const CFString = key;
    let key_void: *const c_void = key_ptr.cast();

    // SAFETY: dict and key are valid references; returned pointer is valid if non-null
    let value_ptr = unsafe { dict.value(key_void) };
    if value_ptr.is_null() {
        return None;
    }

    let cf_type: &CFType = unsafe { &*(value_ptr as *const CFType) };
    let num = cf_type.downcast_ref::<CFNumber>()?;

    let mut result: i64 = 0;
    // SAFETY: result is a valid i64 pointer, CFNumberType::LongLongType matches i64
    let ok = unsafe {
        num.value(CFNumberType::LongLongType, &mut result as *mut i64 as *mut c_void)
    };
    if ok { Some(result) } else { None }
}

/// Extract window bounds from a CFDictionary's kCGWindowBounds key.
/// The bounds value is itself a CFDictionary with X, Y, Width, Height keys.
/// Returns (x, y, width, height) with defaults of 0 on any failure.
#[cfg(target_os = "macos")]
fn get_bounds_from_dict(dict: &objc2_core_foundation::CFDictionary) -> (i32, i32, u32, u32) {
    use core::ffi::c_void;
    use objc2_core_foundation::{CFDictionary, CFString, CFType};
    use objc2_core_graphics::kCGWindowBounds;

    // SAFETY: extern static is valid for the lifetime of the process
    let key_ptr: *const CFString = unsafe { &*kCGWindowBounds };
    let key_void: *const c_void = key_ptr.cast();

    // SAFETY: dict and key are valid references
    let bounds_ptr = unsafe { dict.value(key_void) };
    if bounds_ptr.is_null() {
        return (0, 0, 0, 0);
    }

    let cf_type: &CFType = unsafe { &*(bounds_ptr as *const CFType) };
    let bounds_dict = match cf_type.downcast_ref::<CFDictionary>() {
        Some(d) => d,
        None => return (0, 0, 0, 0),
    };

    let x_key = CFString::from_str("X");
    let y_key = CFString::from_str("Y");
    let w_key = CFString::from_str("Width");
    let h_key = CFString::from_str("Height");

    let x = get_number_from_dict(bounds_dict, &x_key).unwrap_or(0) as i32;
    let y = get_number_from_dict(bounds_dict, &y_key).unwrap_or(0) as i32;
    let width = get_number_from_dict(bounds_dict, &w_key).unwrap_or(0) as u32;
    let height = get_number_from_dict(bounds_dict, &h_key).unwrap_or(0) as u32;

    (x, y, width, height)
}

/// Show a red highlight border around a window using 4 NSWindow overlays.
///
/// Creates 4 borderless, floating NSWindow instances forming a red frame around
/// the target window. Windows auto-dismiss after 3 seconds when the function
/// returns and the overlay windows are dropped.
///
/// # Coordinate Conversion
///
/// macOS uses two coordinate systems:
/// - Quartz (Core Graphics): origin at top-left of screen
/// - Cocoa (AppKit): origin at bottom-left of screen
///
/// This function converts Quartz coordinates from WindowInfo to Cocoa coordinates
/// for NSWindow positioning.
#[cfg(target_os = "macos")]
pub fn show_highlight_border(info: &WindowInfo) -> Result<()> {
    use objc2::{MainThreadOnly, rc::Retained};
    use objc2_app_kit::{
        NSApplication, NSApplicationActivationPolicy, NSColor, NSWindow, NSWindowStyleMask,
    };
    use objc2_core_foundation::CGFloat;
    use objc2_foundation::{NSPoint, NSRect, NSSize, MainThreadMarker};
    use std::thread;
    use std::time::Duration;

    const THICKNESS: CGFloat = 4.0;
    const DISPLAY_SECS: u64 = 3;

    // Initialize NSApplication (required for NSWindow creation)
    // SAFETY: NSApplication sharedApplication is safe to call once per process
    let mtm = MainThreadMarker::new().expect("Must be on main thread");
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    // Get screen height for coordinate conversion (Quartz -> Cocoa)
    // SAFETY: mainScreen returns optional, unwrap with fallback
    let screen_height: CGFloat = {
        use objc2_app_kit::NSScreen;
        let screen = NSScreen::mainScreen(mtm);
        screen.map(|s| s.frame().size.height).unwrap_or(1080.0)
    };

    // Convert Quartz coordinates (top-left origin) to Cocoa (bottom-left origin)
    let quartz_y = info.y as CGFloat;
    let window_height = info.height as CGFloat;
    let cocoa_y = screen_height - (quartz_y + window_height);

    let x = info.x as CGFloat;
    let y = cocoa_y;
    let width = info.width as CGFloat;
    let height = window_height;

    // Calculate the 4 border rectangles
    // Top:    (x, y + height - thickness, width, thickness)
    // Bottom: (x, y, width, thickness)
    // Left:   (x, y + thickness, thickness, height - 2*thickness)
    // Right:  (x + width - thickness, y + thickness, thickness, height - 2*thickness)

    let top_rect = NSRect::new(
        NSPoint::new(x, y + height - THICKNESS),
        NSSize::new(width, THICKNESS),
    );
    let bottom_rect = NSRect::new(
        NSPoint::new(x, y),
        NSSize::new(width, THICKNESS),
    );
    let left_rect = NSRect::new(
        NSPoint::new(x, y + THICKNESS),
        NSSize::new(THICKNESS, height - 2.0 * THICKNESS),
    );
    let right_rect = NSRect::new(
        NSPoint::new(x + width - THICKNESS, y + THICKNESS),
        NSSize::new(THICKNESS, height - 2.0 * THICKNESS),
    );

    let rects = [top_rect, bottom_rect, left_rect, right_rect];

    // Create 4 overlay windows
    let mut windows: Vec<Retained<NSWindow>> = Vec::with_capacity(4);

    for rect in rects {
        // SAFETY: Creating NSWindow with valid frame and style mask
        let window = unsafe {
            let win = NSWindow::initWithContentRect_styleMask_backing_defer(
                NSWindow::alloc(mtm),
                rect,
                NSWindowStyleMask::Borderless,
                objc2_app_kit::NSBackingStoreType::Buffered,
                false,
            );

            // Configure window properties
            win.setOpaque(true);
            win.setBackgroundColor(Some(&NSColor::redColor()));
            win.setIgnoresMouseEvents(true);
            win.setReleasedWhenClosed(false);

            // Set floating window level (kCGFloatingWindowLevel = 3)
            win.setLevel(3);

            win
        };

        windows.push(window);
    }

    // Show all windows
    for window in &windows {
        window.orderFrontRegardless();
    }

    // Keep windows visible for 3 seconds
    thread::sleep(Duration::from_secs(DISPLAY_SECS));

    // Windows are closed/dropped when they go out of scope
    Ok(())
}

/// Stub for non-macOS platforms (prevents compilation errors during development)
#[cfg(not(target_os = "macos"))]
pub fn list_windows() -> Result<Vec<WindowInfo>> {
    anyhow::bail!("macOS platform module is not available on this platform")
}

/// Stub for non-macOS platforms (prevents compilation errors during development)
#[cfg(not(target_os = "macos"))]
pub fn show_highlight_border(_info: &WindowInfo) -> Result<()> {
    anyhow::bail!("macOS highlight border is not available on this platform")
}
