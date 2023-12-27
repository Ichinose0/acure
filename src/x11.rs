use std::{
    ffi::c_ulong,
    mem::MaybeUninit,
    ptr::{null, null_mut},
};

use x11::xlib::{
    Colormap, XAllocColor, XColor, XCreateGC, XDefaultColormap, XFillRectangle, XFlush,
    XGetWindowAttributes, XOpenDisplay, XSetBackground, XSetForeground, XWindowAttributes,
    _XDisplay, _XGC,
};

use crate::{surface::Surface, Color};

pub struct X11Surface {
    display: *mut _XDisplay,
    gc: *mut _XGC,
    window: c_ulong,
}

impl X11Surface {
    pub fn new(window: c_ulong) -> Self {
        unsafe {
            let display = XOpenDisplay(null());

            if display.is_null() {
                panic!("XOpenDisplay failed");
            }
            let gc = XCreateGC(display, window, 0, null_mut());

            Self {
                display,
                gc,
                window,
            }
        }
    }
}

impl Surface for X11Surface {
    fn surface_resize(&mut self, width: u32, height: u32) {}

    fn begin(&mut self) {}

    fn clear(&self, color: crate::Color) {
        unsafe {
            let attributes = get_window_attributes(self.display, self.window);
            XSetForeground(self.display, self.gc, get_color(self.display, color));
            XFillRectangle(
                self.display,
                self.window,
                self.gc,
                0,
                0,
                attributes.width as u32,
                attributes.height as u32,
            );
        }
    }

    fn command(
        &self,
        command: &crate::Command,
        align: crate::AlignMode,
        layout: crate::LayoutMode,
    ) {
        match command {
            crate::Command::FillRectangle(x, y, width, height, radius, color) => unsafe {
                XSetForeground(self.display, self.gc, get_color(self.display, *color));
                XFillRectangle(
                    self.display,
                    self.window,
                    self.gc,
                    *x as i32,
                    *y as i32,
                    *width,
                    *height,
                );
            },
            crate::Command::WriteString(_, _, _, _, _, _) => {}
        }
    }

    fn end(&mut self) {
        unsafe {
            XFlush(self.display);
        }
    }
}

fn get_color(display: *mut _XDisplay, color: Color) -> c_ulong {
    let cmap = unsafe { XDefaultColormap(display, 0) };
    let mut color = match color {
        Color::ARGB(a, r, g, b) => XColor {
            pixel: 0,
            red: (r as u16) * 255,
            green: (r as u16) * 255,
            blue: (r as u16) * 255,
            flags: 0,
            pad: 0,
        },
    };
    unsafe {
        XAllocColor(display, cmap, &mut color);
        color.pixel as c_ulong
    }
}

fn get_window_attributes(display: *mut _XDisplay, window: c_ulong) -> XWindowAttributes {
    let mut attributes = unsafe { MaybeUninit::uninit().assume_init() };
    unsafe { XGetWindowAttributes(display, window, &mut attributes) };
    attributes
}
