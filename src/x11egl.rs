use std::{
    os::raw::c_void,
    ptr::{null, null_mut},
};

use egl::{Config, Context, Display, Instance, Static, Surface};
use std::ffi::c_ulong;
use std::mem::MaybeUninit;
use x11::xlib::{XGetWindowAttributes, XOpenDisplay, XPending, XWindowAttributes, _XDisplay};

pub use khronos_egl as egl;

pub struct Egl {
    instance: Instance<Static>,
    display: Display,
    surface: Surface,
    context: Context,
}

impl Egl {
    pub fn init(window: c_ulong, x_display: *mut _XDisplay) -> Self {
        let instance = egl::Instance::new(egl::Static);
        unsafe {
            let display = match instance.get_display(x_display as egl::NativeDisplayType) {
                Some(d) => d,
                None => panic!("Can't get EGLDisplay"),
            };
            match instance.initialize(display) {
                Ok(_) => {}
                Err(e) => panic!("{:?}", e),
            };

            let attr = vec![
                egl::BUFFER_SIZE,
                16,
                egl::RED_SIZE,
                8,
                egl::GREEN_SIZE,
                8,
                egl::BLUE_SIZE,
                8,
                egl::ALPHA_SIZE,
                8,
                egl::SURFACE_TYPE,
                egl::WINDOW_BIT,
                egl::RENDERABLE_TYPE,
                egl::OPENGL_ES2_BIT,
                egl::NONE,
            ];

            let mut configs = Vec::with_capacity(1);

            instance
                .choose_config(display, &attr, &mut configs)
                .unwrap();

            if configs.len() != 1 {
                panic!("Can't choose context config");
            }

            let surface = instance
                .create_window_surface(display, configs[0], window as egl::NativeWindowType, None)
                .unwrap();
            let ctx_attr = vec![egl::CONTEXT_CLIENT_VERSION, 2, egl::NONE];
            let context = instance
                .create_context(display, configs[0], None, &ctx_attr)
                .unwrap();

            Self {
                instance,
                display,
                surface,
                context,
            }
        }
    }

    pub fn make_current(&self) {
        self.instance.make_current(
            self.display,
            Some(self.surface),
            Some(self.surface),
            Some(self.context),
        );
    }

    pub fn swap_intervals(&self, interval: bool) {
        self.instance.swap_interval(self.display, interval as i32);
    }

    pub fn swap_buffers(&self) {
        self.instance
            .swap_buffers(self.display, self.surface)
            .unwrap();
    }

    pub fn get_proc_address(&self, procname: &str) -> *const c_void {
        self.instance.get_proc_address(procname).unwrap() as *const c_void
    }
}

impl Drop for Egl {
    fn drop(&mut self) {
        self.instance
            .destroy_context(self.display, self.context)
            .unwrap();
        self.instance
            .destroy_surface(self.display, self.surface)
            .unwrap();
        self.instance.terminate(self.display).unwrap();
    }
}

pub struct X11EglSurface {
    display: *mut _XDisplay,
    window: c_ulong,
    egl: Egl,
}

impl X11EglSurface {
    pub fn new(window: c_ulong) -> Self {
        unsafe {
            let display = XOpenDisplay(null());

            if display.is_null() {
                panic!("XOpenDisplay failed");
            }

            let egl = Egl::init(window, display);
            egl.swap_intervals(true);
            egl.make_current();

            gl::load_with(|s| egl.get_proc_address(s));

            Self {
                display,
                window,
                egl,
            }
        }
    }
}

impl crate::Surface for X11EglSurface {
    fn surface_resize(&mut self, width: u32, height: u32) {}

    fn begin(&mut self) {
        unsafe {
            XPending(self.display);
        }
    }

    fn clear(&self, color: crate::Color) {
        unsafe {
            match color {
                crate::Color::ARGB(a, r, g, b) => {
                    gl::ClearColor(
                        (r as f32) / 255.0,
                        (g as f32) / 255.0,
                        (b as f32) / 255.0,
                        (a as f32) / 255.0,
                    );
                }
            }
        }
    }

    fn command(
        &self,
        command: &crate::Command,
        align: crate::AlignMode,
        layout: crate::LayoutMode,
    ) {
        match command {
            crate::Command::FillRectangle(x, y, width, height, radius, color) => {}

            crate::Command::WriteString(x, y, width, height, color, text) => {}
        }
    }

    fn end(&mut self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        self.egl.swap_buffers();
    }
}

fn get_window_attributes(display: *mut _XDisplay, window: c_ulong) -> XWindowAttributes {
    let mut attributes = unsafe { MaybeUninit::uninit().assume_init() };
    unsafe { XGetWindowAttributes(display, window, &mut attributes) };
    attributes
}
