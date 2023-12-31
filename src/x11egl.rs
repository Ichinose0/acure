const FRAGMENT: &'static str = include_str!("shader/shader.frag");
const VERTEX: &'static str = include_str!("shader/shader.vert");

use std::{
    os::raw::c_void,
    ptr::{null, null_mut}, ffi::{CString, CStr},
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
    context: Context
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
    vertex: u32,
    fragment: u32,
    program: u32
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

            let vertex = gl::CreateShader(gl::VERTEX_SHADER);
            let fragment = gl::CreateShader(gl::FRAGMENT_SHADER);

            gl::ShaderSource(vertex,1,&CString::new(VERTEX).unwrap().as_ptr(),null());
            gl::CompileShader(vertex);

            gl::ShaderSource(fragment,1,&CString::new(FRAGMENT).unwrap().as_ptr(),null());
            gl::CompileShader(fragment);

            let program = gl::CreateProgram();

            gl::AttachShader(program,vertex);
            gl::AttachShader(program,fragment);
            gl::LinkProgram(program);

            let mut result = 0;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut result);

            if (result as u8) == gl::FALSE {
                let mut length = 0;
                gl::GetProgramiv(program,gl::INFO_LOG_LENGTH,&mut length);
                let mut log = Vec::with_capacity(length as usize);
                gl::GetProgramInfoLog(program,length,null_mut(),log.as_mut_ptr());
                let log = CStr::from_ptr(log.as_ptr());
                let log_str = log.to_str().unwrap();
                panic!("{:#?}",log_str);
            }

            gl::UseProgram(program);

            Self {
                display,
                window,
                egl,
                vertex,
                fragment,
                program
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
            gl::Clear(gl::COLOR_BUFFER_BIT|gl::DEPTH_BUFFER_BIT);
            gl::ClearDepth(1.0);
        }
    }

    fn command(
        &self,
        command: &crate::Command,
        align: crate::AlignMode,
        layout: crate::LayoutMode,
    ) {
        
        match command {
            crate::Command::FillRectangle(x, y, width, height, radius, color) => {
                unsafe {
                    let position: Vec<f32> = vec![
                        -0.5,0.5,
                        -0.5,-0.5,
                        0.5,-0.5,
                        0.5,0.5
                    ];
                    let vert_color: Vec<f32>;
                    match color {
                        crate::Color::ARGB(a,r,g,b) => {
                            let a = (*a as f32)/255.0;
                            let r = (*r as f32)/255.0;
                            let g = (*g as f32)/255.0;
                            let b = (*b as f32)/255.0;
                            vert_color = vec![
                                r,g,b,a,
                                r,g,b,a,
                                r,g,b,a,
                                r,g,b,a
                            ];
                        }
                    }
                    let att_location = gl::GetAttribLocation(self.program,CString::new("position").unwrap().as_ptr());
                    let color_location = gl::GetAttribLocation(self.program,CString::new("color").unwrap().as_ptr());
                    gl::EnableVertexAttribArray(att_location as u32);
                    gl::EnableVertexAttribArray(color_location as u32);
                    gl::VertexAttribPointer(att_location as u32,2,gl::FLOAT,0,0,position.as_ptr() as *const c_void);
                    gl::VertexAttribPointer(color_location as u32,4,gl::FLOAT,0,0,vert_color.as_ptr() as *const c_void);
                    gl::DrawArrays(gl::TRIANGLE_FAN,0,4);
                }
            }

            crate::Command::WriteString(x, y, width, height, color, text) => {}
        }
    }

    fn end(&mut self) {
        self.egl.swap_buffers();
    }
}

fn get_window_attributes(display: *mut _XDisplay, window: c_ulong) -> XWindowAttributes {
    let mut attributes = unsafe { MaybeUninit::uninit().assume_init() };
    unsafe { XGetWindowAttributes(display, window, &mut attributes) };
    attributes
}
