//! Wayland layer-shell highlight implementation
//!
//! Uses zwlr_layer_shell_v1 protocol to create overlay borders around windows.
//! Works on wlroots-based compositors (Sway, Hyprland, etc.).

use anyhow::{Context, Result};
use crate::error::AppError;
use crate::window::WindowInfo;

use std::os::unix::fs::MetadataExt;
use std::fs::File;
use std::thread;
use std::time::Duration;
use memmap2::MmapMut;

use wayland_client::{
    Connection, Dispatch, QueueHandle,
    protocol::{wl_registry, wl_compositor, wl_shm, wl_surface, wl_buffer, wl_region, wl_shm_pool},
    globals::GlobalListContents,
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{ZwlrLayerShellV1, Layer},
    zwlr_layer_surface_v1::{ZwlrLayerSurfaceV1, self, Anchor, KeyboardInteractivity},
};

/// State for layer-shell highlight
struct HighlightState {
    compositor: Option<wl_compositor::WlCompositor>,
    shm: Option<wl_shm::WlShm>,
    layer_shell: Option<ZwlrLayerShellV1>,
    surfaces: Vec<LayerSurface>,
    configured: bool,
}

struct LayerSurface {
    surface: wl_surface::WlSurface,
    layer_surface: ZwlrLayerSurfaceV1,
    width: u32,
    height: u32,
}

impl HighlightState {
    fn new() -> Self {
        Self {
            compositor: None,
            shm: None,
            layer_shell: None,
            surfaces: Vec::new(),
            configured: false,
        }
    }
}

/// Show a red highlight border around a window using layer-shell protocol
///
/// Creates 4 overlay surfaces (top, bottom, left, right) using layer-shell.
/// Surfaces are positioned using anchor and margin to form a border around
/// the target window geometry.
///
/// # Arguments
/// * `connection` - Wayland connection
/// * `info` - WindowInfo with geometry (x, y, width, height)
///
/// # Errors
/// Returns error if:
/// - Layer-shell protocol not available
/// - Window has no geometry (x=y=width=height=0)
/// - Compositor doesn't support layer-shell
pub fn show_highlight_border_layer_shell(
    connection: &Connection,
    info: &WindowInfo,
) -> Result<()> {
    // Check if we have valid geometry
    if info.width == 0 || info.height == 0 {
        return Err(AppError::enumeration_failed(
            "Cannot highlight window: geometry not available on Wayland (foreign-toplevel doesn't provide position/size)"
        ).into());
    }

    let display = connection.display();
    let mut event_queue = connection.new_event_queue();
    let qh = event_queue.handle();

    let registry = display.get_registry(&qh, ());

    let mut state = HighlightState::new();

    // Roundtrip to bind globals
    event_queue.roundtrip(&mut state)
        .map_err(|e| AppError::platform_error(format!("Wayland roundtrip failed: {}", e)))?;

    // Check for required globals
    let layer_shell = state.layer_shell.ok_or_else(|| {
        AppError::enumeration_failed(
            "Layer-shell protocol not available. Highlight requires a wlroots-based compositor (Sway, Hyprland, etc.)"
        )
    })?;

    let compositor = state.compositor.ok_or_else(|| {
        AppError::platform_error("Compositor global not available")
    })?;

    let shm = state.shm.ok_or_else(|| {
        AppError::platform_error("SHM global not available")
    })?;

    // Create 4 border surfaces
    create_border_surfaces(
        &mut state,
        &layer_shell,
        &compositor,
        &qh,
        info.x,
        info.y,
        info.width,
        info.height,
    )?;

    // Process events until all surfaces are configured
    for _ in 0..10 {
        event_queue.dispatch_pending(&mut state)
            .map_err(|e| AppError::platform_error(format!("Event dispatch failed: {}", e)))?;

        if state.configured {
            break;
        }

        event_queue.flush()
            .map_err(|e| AppError::platform_error(format!("Flush failed: {}", e)))?;

        if let Some(guard) = event_queue.prepare_read() {
            match guard.read() {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(AppError::platform_error(format!("Read failed: {}", e)).into()),
            }
        }
    }

    // Attach buffers and commit surfaces
    for layer_surface in &state.surfaces {
        let buffer = create_red_buffer(&shm, layer_surface.width, layer_surface.height, &qh)?;
        layer_surface.surface.attach(Some(&buffer), 0, 0);
        layer_surface.surface.commit();
    }

    // Display for 3 seconds
    thread::sleep(Duration::from_secs(3));

    // Cleanup: surfaces are destroyed when dropped
    Ok(())
}

fn create_border_surfaces(
    state: &mut HighlightState,
    layer_shell: &ZwlrLayerShellV1,
    compositor: &wl_compositor::WlCompositor,
    qh: &QueueHandle<HighlightState>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<()> {
    const THICKNESS: u32 = 4;

    // Define 4 border segments: (anchor, margin_left, margin_top, width, height)
    let segments = [
        // Top
        (Anchor::Top, x, y, width, THICKNESS),
        // Bottom
        (Anchor::Bottom, x, y, width, THICKNESS),
        // Left
        (Anchor::Left, x, y + THICKNESS as i32, THICKNESS, height - 2 * THICKNESS),
        // Right
        (Anchor::Right, x + width as i32 - THICKNESS as i32, y + THICKNESS as i32, THICKNESS, height - 2 * THICKNESS),
    ];

    for (anchor, margin_left, margin_top, seg_width, seg_height) in segments {
        let surface = compositor.create_surface(qh, ());

        let layer_surface = layer_shell.get_layer_surface(
            &surface,
            None, // Let compositor choose output
            Layer::Overlay,
            "snap-window-highlight".to_string(),
            qh,
            (),
        );

        // Configure layer surface
        layer_surface.set_size(seg_width, seg_height);
        layer_surface.set_anchor(anchor | Anchor::Left | Anchor::Top);
        layer_surface.set_margin(margin_top, 0, 0, margin_left);
        layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);

        // Set empty input region for click-through
        let region = compositor.create_region(qh, ());
        surface.set_input_region(Some(&region));
        region.destroy();

        surface.commit();

        state.surfaces.push(LayerSurface {
            surface,
            layer_surface,
            width: seg_width,
            height: seg_height,
        });
    }

    Ok(())
}

fn create_red_buffer(
    shm: &wl_shm::WlShm,
    width: u32,
    height: u32,
    qh: &QueueHandle<HighlightState>,
) -> Result<wl_buffer::WlBuffer> {
    let stride = width * 4; // ARGB8888 = 4 bytes per pixel
    let size = (stride * height) as usize;

    // Create shared memory file
    let fd = create_shm_file(size)?;
    let mut mmap = unsafe { MmapMut::map_mut(&fd).context("Failed to mmap shared memory")? };

    // Fill with opaque red (ARGB: 0xFF0000FF)
    // Note: Wayland ARGB8888 is native endian, so on little-endian: [B, G, R, A]
    for chunk in mmap.chunks_exact_mut(4) {
        chunk[0] = 0x00; // Blue
        chunk[1] = 0x00; // Green
        chunk[2] = 0xFF; // Red
        chunk[3] = 0xFF; // Alpha (opaque)
    }

    mmap.flush().context("Failed to flush mmap")?;

    // Create pool and buffer
    let pool = shm.create_pool(fd.as_raw_fd(), size as i32, qh, ());
    let buffer = pool.create_buffer(0, width as i32, height as i32, stride as i32, wl_shm::Format::Argb8888, qh, ());

    // Pool can be destroyed after creating buffer (buffer keeps reference)
    pool.destroy();

    Ok(buffer)
}

fn create_shm_file(size: usize) -> Result<File> {
    use std::env::temp_dir;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let path = temp_dir().join(format!("wayland-shm-{}-{}", process::id(), timestamp));
    let file = File::create(&path).context("Failed to create shm file")?;
    file.set_len(size as u64).context("Failed to set shm size")?;

    // Unlink immediately so file is cleaned up on close
    let _ = std::fs::remove_file(&path);

    Ok(file)
}

// Dispatch implementations
impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for HighlightState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global { name, interface, version } = event {
            if interface == ZwlrLayerShellV1::interface().name {
                state.layer_shell = Some(registry.bind(name, version.min(4), qh, ()));
            } else if interface == wl_compositor::WlCompositor::interface().name {
                state.compositor = Some(registry.bind(name, version.min(4), qh, ()));
            } else if interface == wl_shm::WlShm::interface().name {
                state.shm = Some(registry.bind(name, version.min(1), qh, ()));
            }
        }
    }
}

impl Dispatch<ZwlrLayerSurfaceV1, ()> for HighlightState {
    fn event(
        state: &mut Self,
        surface: &ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_layer_surface_v1::Event::Configure { serial, .. } => {
                surface.ack_configure(serial);
                state.configured = true;
            }
            zwlr_layer_surface_v1::Event::Closed => {
                // Surface closed by compositor
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for HighlightState {
    fn event(_: &mut Self, _: &wl_surface::WlSurface, _: wl_surface::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

impl Dispatch<wl_buffer::WlBuffer, ()> for HighlightState {
    fn event(_: &mut Self, _: &wl_buffer::WlBuffer, _: wl_buffer::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

impl Dispatch<wl_shm::WlShm, ()> for HighlightState {
    fn event(_: &mut Self, _: &wl_shm::WlShm, _: wl_shm::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for HighlightState {
    fn event(_: &mut Self, _: &wl_shm_pool::WlShmPool, _: wl_shm_pool::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}
