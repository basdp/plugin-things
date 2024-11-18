use cursor_icon::CursorIcon;
use raw_window_handle::{RawWindowHandle, HasWindowHandle, HasDisplayHandle};

use crate::{platform::{window::OsWindow, interface::{OsWindowInterface, OsWindowHandle}}, error::Error, event::EventCallback, dimensions::LogicalSize, LogicalPosition};

#[derive(Clone)]
pub struct WindowAttributes {
    pub(crate) size: LogicalSize,
    pub(crate) scale: f64,
}

impl WindowAttributes {
    pub fn new(size: LogicalSize, scale: f64) -> Self {
        Self {
            size,
            scale,
        }
    }

    pub fn with_size(size: LogicalSize) -> Self {
        Self::new(size, 1.0)
    }

    pub fn size(&self) -> LogicalSize {
        self.size
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn scaled_size(&self) -> LogicalSize {
        self.size * self.scale
    }
}

pub struct Window {
    attributes: WindowAttributes,
    os_window_handle: OsWindowHandle,
}

impl Window {
    pub fn open(
        parent: RawWindowHandle,
        attributes: WindowAttributes,
        event_callback: Box<EventCallback>,
    ) -> Result<Window, Error> {
        let os_window_handle = OsWindow::open(
            parent,
            attributes.clone(),
            event_callback,
        )?;

        Ok(Self {
            attributes,
            os_window_handle,
        })
    }

    pub fn attributes(&self) -> &WindowAttributes {
        &self.attributes
    }

    pub fn os_scale(&self) -> f64 {
        self.os_window_handle.window().os_scale()
    }

    /// This only needs to be called on Linux
    pub fn poll_events(&self) -> Result<(), Error> {
        self.os_window_handle.window().poll_events()
    }

    pub fn set_cursor(&self, cursor: Option<CursorIcon>) {
        self.os_window_handle.window().set_cursor(cursor);
    }

    pub fn set_input_focus(&self, focus: bool) {
        self.os_window_handle.window().set_input_focus(focus);
    }

    pub fn warp_mouse(&self, position: LogicalPosition) {
        self.os_window_handle.window().warp_mouse(position);
    }

    pub fn set_size(&self, size: LogicalSize) {
        self.os_window_handle.window().set_size(size);
    }
}

impl HasWindowHandle for Window {
    fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        self.os_window_handle.window_handle()
    }
}

impl HasDisplayHandle for Window {
    fn display_handle(&self) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        self.os_window_handle.display_handle()
    }
}
