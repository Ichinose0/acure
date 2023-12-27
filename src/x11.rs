use std::{
    ffi::c_ulong,
    ptr::{null, null_mut},
};

use x11::xlib::{
    Colormap, XAllocColor, XColor, XCreateGC, XOpenDisplay, XSetBackground, _XDisplay, _XGC,
};

use crate::{surface::Surface, Color};

pub struct X11Surface {
    display: *mut _XDisplay,
    gc: *mut _XGC,
    window: u64,
}

impl X11Surface {
    pub fn new(window: u64) -> Self {
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
            XSetBackground(self.display, self.gc, get_color(self.display, color));
        }
    }

    fn command(
        &self,
        command: &crate::Command,
        align: crate::AlignMode,
        layout: crate::LayoutMode,
    ) {
    }

    fn end(&mut self) {}
}

fn get_color(display: *mut _XDisplay, color: Color) -> c_ulong {
    let cmap = Colormap::default();
    let mut color = match color {
        Color::ARGB(a, r, g, b) => XColor {
            pixel: 0,
            red: r as u16,
            green: g as u16,
            blue: b as u16,
            flags: 0,
            pad: 0,
        },
    };
    unsafe {
        XAllocColor(display, cmap, &mut color);
        color.pixel as c_ulong
    }
}
