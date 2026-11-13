//! Keeping the figure on the desktop, like the icons and folders there.
//!
//! On X11 this module talks to the X server directly (one connection, kept
//! open) to:
//! * ask the window manager, through standard EWMH requests, to keep the
//!   window below every other window, out of the taskbar and the workspace
//!   switcher, and on all workspaces;
//! * set the window's *input shape*, so the transparent parts let clicks
//!   through to the desktop icons behind them;
//! * read the pointer's position on screen, for smooth dragging.
//!
//! On Wayland none of this is available; [`Desktop::connect`] returns `None`
//! and the app falls back to the compositor's own window moving.

use eframe::egui::{Pos2, Rect, pos2};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use x11rb::connection::Connection;
use x11rb::protocol::shape::{ConnectionExt as _, SK, SO};
use x11rb::protocol::xproto::{ClientMessageEvent, ClipOrdering, ConnectionExt as _, EventMask, Rectangle};
use x11rb::rust_connection::RustConnection;

const REMOVE: u32 = 0;
const ADD: u32 = 1;
/// "Source indication": a normal application.
const SOURCE_APP: u32 = 1;
const ALL_DESKTOPS: u32 = 0xFFFF_FFFF;

pub struct Desktop {
    conn: RustConnection,
    root: u32,
    window: u32,
    /// Last input shape sent, in physical pixels, to avoid resending it.
    shape: Vec<(i16, i16, u16, u16)>,
}

impl Desktop {
    /// Connect when the window is an X11 window.
    pub fn connect(frame: &eframe::Frame) -> Option<Desktop> {
        let handle = frame.window_handle().ok()?;
        let window: u32 = match handle.as_raw() {
            RawWindowHandle::Xlib(h) => h.window as u32,
            RawWindowHandle::Xcb(h) => h.window.get(),
            _ => return None,
        };
        let (conn, screen) = x11rb::connect(None).ok()?;
        let root = conn.setup().roots[screen].root;
        Some(Desktop { conn, root, window, shape: Vec::new() })
    }

    fn atom(&self, name: &str) -> Result<u32, String> {
        Ok(self
            .conn
            .intern_atom(false, name.as_bytes())
            .map_err(|e| e.to_string())?
            .reply()
            .map_err(|e| e.to_string())?
            .atom)
    }

    fn send(&self, kind: u32, data: [u32; 5]) -> Result<(), String> {
        let ev = ClientMessageEvent::new(32, self.window, kind, data);
        self.conn
            .send_event(false, self.root, EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY, ev)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Apply the desktop-widget hints. `above` floats the figure over other
    /// windows instead of keeping it on the desktop.
    pub fn pin(&self, above: bool) -> Result<(), String> {
        let state = self.atom("_NET_WM_STATE")?;
        let (below, above_atom) = (self.atom("_NET_WM_STATE_BELOW")?, self.atom("_NET_WM_STATE_ABOVE")?);
        self.send(state, [ADD, self.atom("_NET_WM_STATE_SKIP_TASKBAR")?, self.atom("_NET_WM_STATE_SKIP_PAGER")?, SOURCE_APP, 0])?;
        self.send(state, [ADD, self.atom("_NET_WM_STATE_STICKY")?, 0, SOURCE_APP, 0])?;
        let (on, off) = if above { (above_atom, below) } else { (below, above_atom) };
        self.send(state, [REMOVE, off, 0, SOURCE_APP, 0])?;
        self.send(state, [ADD, on, 0, SOURCE_APP, 0])?;
        self.send(self.atom("_NET_WM_DESKTOP")?, [ALL_DESKTOPS, SOURCE_APP, 0, 0, 0])?;
        self.conn.flush().map_err(|e| e.to_string())
    }

    /// Only these rectangles (logical points, window coordinates) receive
    /// the pointer; clicks anywhere else go to whatever is beneath.
    pub fn set_input_region(&mut self, rects: &[Rect], pixels_per_point: f32) -> Result<(), String> {
        let shape: Vec<Rectangle> = rects
            .iter()
            .map(|r| {
                let r = Rect::from_min_max(
                    pos2((r.min.x * pixels_per_point).floor(), (r.min.y * pixels_per_point).floor()),
                    pos2((r.max.x * pixels_per_point).ceil(), (r.max.y * pixels_per_point).ceil()),
                );
                Rectangle {
                    x: r.min.x.max(0.0) as i16,
                    y: r.min.y.max(0.0) as i16,
                    width: r.width().max(1.0) as u16,
                    height: r.height().max(1.0) as u16,
                }
            })
            .collect();
        let key: Vec<_> = shape.iter().map(|r| (r.x, r.y, r.width, r.height)).collect();
        if key == self.shape {
            return Ok(());
        }
        self.conn
            .shape_rectangles(SO::SET, SK::INPUT, ClipOrdering::UNSORTED, self.window, 0, 0, &shape)
            .map_err(|e| e.to_string())?;
        self.conn.flush().map_err(|e| e.to_string())?;
        self.shape = key;
        Ok(())
    }

    /// The pointer's position on screen, in logical points.
    pub fn pointer(&self, pixels_per_point: f32) -> Option<Pos2> {
        let reply = self.conn.query_pointer(self.root).ok()?.reply().ok()?;
        Some(pos2(f32::from(reply.root_x), f32::from(reply.root_y)) / pixels_per_point)
    }

    /// The window's position on screen, in logical points.
    pub fn window_position(&self, pixels_per_point: f32) -> Option<Pos2> {
        let t = self.conn.translate_coordinates(self.window, self.root, 0, 0).ok()?.reply().ok()?;
        Some(pos2(f32::from(t.dst_x), f32::from(t.dst_y)) / pixels_per_point)
    }
}
