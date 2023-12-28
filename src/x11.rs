use std::{
    ffi::{c_ulong, CString},
    mem::MaybeUninit,
    ptr::{null, null_mut},
};

use x11::{
    xft::{
        XftColorAllocValue, XftColorFree, XftDraw, XftDrawCreate, XftDrawStringUtf8,
        XftFontOpenName, XftTextExtentsUtf8,
    },
    xlib::{
        Colormap, Visual, XAllocColor, XColor, XCreateGC, XDefaultColormap, XDefaultScreen,
        XDefaultVisual, XFillRectangle, XFlush, XGetWindowAttributes, XOpenDisplay, XSetBackground,
        XSetForeground, XWindowAttributes, _XDisplay, _XGC, XFreeColormap, XFreeColors,
    },
    xrender::XRenderColor,
};

use crate::{surface::Surface, Color};

pub struct XftColor {
    display: *mut _XDisplay,
    inner: x11::xft::XftColor,
}

impl XftColor {
    pub fn alloc(display: *mut _XDisplay, color: Color) -> Self {
        unsafe {
            let mut inner = unsafe { MaybeUninit::uninit().assume_init() };
            let color = match color {
                Color::ARGB(a, r, g, b) => XRenderColor {
                    red: (r as u16) * 255,
                    green: (g as u16) * 255,
                    blue: (b as u16) * 255,
                    alpha: (a as u16) * 255,
                },
            };

            XftColorAllocValue(
                display,
                XDefaultVisual(display, XDefaultScreen(display)),
                XDefaultColormap(display, XDefaultScreen(display)),
                &color,
                &mut inner,
            );
            Self { display, inner }
        }
    }
}

impl Drop for XftColor {
    fn drop(&mut self) {
        unsafe {
            XftColorFree(
                self.display,
                XDefaultVisual(self.display, XDefaultScreen(self.display)),
                XDefaultColormap(self.display, XDefaultScreen(self.display)),
                &mut self.inner,
            );
        }
    }
}

pub struct X11Surface {
    display: *mut _XDisplay,
    gc: *mut _XGC,
    xft: *mut XftDraw,
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
            let xft = XftDrawCreate(
                display,
                window,
                XDefaultVisual(display, XDefaultScreen(display)),
                XDefaultColormap(display, XDefaultScreen(display)),
            );

            Self {
                display,
                gc,
                xft,
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
            crate::Command::WriteString(x, y, width, height, color, text) => {
                let fontname = CString::new("Yu gothic-12").unwrap();
                let font = unsafe {
                    XftFontOpenName(
                        self.display,
                        XDefaultScreen(self.display),
                        fontname.as_ptr(),
                    )
                };
                unsafe {
                    let color = XftColor::alloc(self.display, *color);
                    XftDrawStringUtf8(
                        self.xft,
                        &color.inner,
                        font,
                        *x as i32,
                        (*y as i32) + (*font).ascent,
                        text.as_ptr(),
                        text.len() as i32,
                    );
                }
            }
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
        XFreeColormap(display, cmap);
        color.pixel as c_ulong
    }
}

fn get_window_attributes(display: *mut _XDisplay, window: c_ulong) -> XWindowAttributes {
    let mut attributes = unsafe { MaybeUninit::uninit().assume_init() };
    unsafe { XGetWindowAttributes(display, window, &mut attributes) };
    attributes
}
