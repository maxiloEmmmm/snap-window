//! snap-window - Cross-platform CLI window screenshot tool
//!
//! This crate provides functionality for capturing screenshots of specific
//! application windows across Windows, macOS, and Linux platforms.

pub mod cli;
pub mod error;
pub mod platform;
pub mod window;
pub mod window_service;
pub mod capture_service;
