//! Platform-specific window enumeration
//!
//! This module provides cross-platform window enumeration using conditional compilation.
//! Each platform (Windows, macOS, Linux) has its own implementation module that is
//! compiled only when targeting that platform.
//!
//! # Supported Platforms
//!
//! - **Windows**: Uses Windows API via windows-rs crate (future implementation)
//! - **macOS**: Uses Core Graphics via objc2-core-graphics (future implementation)
//! - **Linux**: Uses X11 via x11rb crate with Wayland fallback (future implementation)
//!
//! # Adding New Platforms
//!
//! To add support for a new platform:
//! 1. Create a new file: `src/platform/{platform}.rs`
//! 2. Add `#[cfg(target_os = "...")]` attributes
//! 3. Implement the `list_windows()` function
//! 4. Add conditional imports below
//!
//! # Conditional Compilation
//!
//! The `#[cfg(target_os = "...")]` attribute ensures only the relevant platform
//! module is compiled for each target. This prevents compilation errors from
//! platform-specific APIs on unsupported platforms.

// Windows platform support
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::list_windows;

// macOS platform support
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::list_windows;

// Linux platform support
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::list_windows;

// Unsupported platform fallback
// This provides a compile-time error for unsupported platforms
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
compile_error!("Unsupported platform. Only Windows, macOS, and Linux are supported.");
