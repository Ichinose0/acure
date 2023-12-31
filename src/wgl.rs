const FRAGMENT: &'static str = include_str!("shader/shader.frag");
const VERTEX: &'static str = include_str!("shader/shader.vert");

use std::{
    ffi::{c_void, CString},
    mem::{size_of, MaybeUninit},
    ptr::{null, null_mut},
};

use windows::{
    core::PCSTR,
    Win32::{
        Foundation::HWND,
        Graphics::{Gdi::{GetDC, HDC}, OpenGL::*},
    },
};

use crate::gl::{compile_shader, create_program};

type wglCreateContextAttribsARB =
    fn(hDC: *mut c_void, hshareContext: *mut c_void, attribList: *const i32) -> *mut c_void;

const WGL_CONTEXT_MAJOR_VERSION_ARB:i32 =         0x2091;
const WGL_CONTEXT_MINOR_VERSION_ARB:i32 =         0x2092;
const WGL_CONTEXT_LAYER_PLANE_ARB:i32   =         0x2093;
const WGL_CONTEXT_FLAGS_ARB:i32         =         0x2094;
const WGL_CONTEXT_PROFILE_MASK_ARB:i32  =         0x9126;

const WGL_CONTEXT_DEBUG_BIT_ARB:i32              = 0x0001;
const WGL_CONTEXT_FORWARD_COMPATIBLE_BIT_ARB:i32 = 0x0002;


const WGL_CONTEXT_CORE_PROFILE_BIT_ARB:i32          = 0x00000001;
const WGL_CONTEXT_COMPATIBILITY_PROFILE_BIT_ARB:i32 = 0x00000002;

const ERROR_INVALID_VERSION_ARB:i32       =       0x2095;
const ERROR_INVALID_PROFILE_ARB:i32       =       0x2096;

pub struct Wgl {
    wglCreateContextAttribsARB: wglCreateContextAttribsARB,
    hdc: HDC
}

impl Wgl {
    #[inline]
    pub fn load_with<F>(hwnd: HWND, func: F) -> Self
    where
        F: Fn(&str) -> *const c_void,
    {
        let mut hdc = unsafe { GetDC(hwnd) };
        let pfd = PIXELFORMATDESCRIPTOR {
            nSize: size_of::<PIXELFORMATDESCRIPTOR>() as u16,
            nVersion: 1,
            dwFlags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
            iPixelType: PFD_TYPE_RGBA,
            cColorBits: 32,
            cRedBits: 0,
            cRedShift: 0,
            cGreenBits: 0,
            cGreenShift: 0,
            cBlueBits: 0,
            cBlueShift: 0,
            cAlphaBits: 0,
            cAlphaShift: 0,
            cAccumBits: 0,
            cAccumRedBits: 0,
            cAccumGreenBits: 0,
            cAccumBlueBits: 0,
            cAccumAlphaBits: 0,
            cDepthBits: 24,
            cStencilBits: 8,
            cAuxBuffers: 0,
            iLayerType: 0,
            bReserved: 0,
            dwLayerMask: 0,
            dwVisibleMask: 0,
            dwDamageMask: 0,
        };

        let pixel_format = unsafe { ChoosePixelFormat(hdc, &pfd) };
        let ctx = unsafe {
            SetPixelFormat(hdc, pixel_format, &pfd);
            let ctx = wglCreateContext(hdc).unwrap();
            wglMakeCurrent(hdc, ctx);
            ctx
        };

        let attribs = [
            WGL_CONTEXT_MAJOR_VERSION_ARB,
            3,
            WGL_CONTEXT_MINOR_VERSION_ARB,
            3,
            WGL_CONTEXT_PROFILE_MASK_ARB,
            WGL_CONTEXT_CORE_PROFILE_BIT_ARB,
            0,
        ];

        let wglCreateContextAttribsARB: wglCreateContextAttribsARB =
            unsafe { std::mem::transmute(func("wglCreateContextAttribsARB")) };
        let new_ctx = HGLRC((wglCreateContextAttribsARB)(hdc.0 as *mut c_void, null_mut(), attribs.as_ptr()) as isize);
        unsafe {
            wglDeleteContext(ctx);
            wglMakeCurrent(hdc, new_ctx);
        }

        Self {
            wglCreateContextAttribsARB,
            hdc
        }
    }

    #[inline]
    pub fn make_current(&self) {}

    #[inline]
    pub fn swap_intervals(&self, interval: bool) {}

    #[inline]
    pub fn swap_buffers(&self) {
        unsafe {
            SwapBuffers(self.hdc).unwrap();
        }
    }

    #[inline]
    pub fn get_proc_address(&self, procname: &str) -> *const c_void {
        unsafe {
            wglGetProcAddress(PCSTR(format!("{}\0",procname).as_ptr())).unwrap() as *const c_void
        }
    }
}

impl Drop for Wgl {
    fn drop(&mut self) {}
}

pub struct WglSurface {
    hwnd: HWND,
    wgl: Wgl,
    vertex: u32,
    fragment: u32,
    program: u32,
}

impl WglSurface {
    #[inline]
    pub fn new(window: isize) -> Self {
        let hwnd = HWND(window);
        unsafe {
            let wgl = Wgl::load_with(hwnd, |s| {
                wglGetProcAddress(PCSTR(format!("{}\0", s).as_ptr())).unwrap() as *const c_void
            });

            gl::load_with(|s| {
                wgl.get_proc_address(s)
            });

            let vertex = compile_shader(gl::VERTEX_SHADER, VERTEX);
            let fragment = compile_shader(gl::FRAGMENT_SHADER, FRAGMENT);

            let program = create_program(&[vertex, fragment]);
            gl::UseProgram(program);

            Self {
                hwnd,
                wgl,
                vertex,
                fragment,
                program,
            }
        }
    }
}

impl crate::Surface for WglSurface {
    #[inline]
    fn surface_resize(&mut self, width: u32, height: u32) {}

    #[inline]
    fn begin(&mut self) {}

    #[inline]
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
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::ClearDepth(1.0);
        }
    }

    #[inline]
    fn command(
        &self,
        command: &crate::Command,
        align: crate::AlignMode,
        layout: crate::LayoutMode,
    ) {
        match command {
            crate::Command::FillRectangle(x, y, width, height, radius, color) => unsafe {
                let position: Vec<f32> = vec![-0.5, 0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5];
                let vert_color: Vec<f32>;
                match color {
                    crate::Color::ARGB(a, r, g, b) => {
                        let a = (*a as f32) / 255.0;
                        let r = (*r as f32) / 255.0;
                        let g = (*g as f32) / 255.0;
                        let b = (*b as f32) / 255.0;
                        vert_color = vec![r, g, b, a, r, g, b, a, r, g, b, a, r, g, b, a];
                    }
                }
                let att_location =
                    gl::GetAttribLocation(self.program, CString::new("position").unwrap().as_ptr());
                let color_location =
                    gl::GetAttribLocation(self.program, CString::new("color").unwrap().as_ptr());
                gl::EnableVertexAttribArray(att_location as u32);
                gl::EnableVertexAttribArray(color_location as u32);
                gl::VertexAttribPointer(
                    att_location as u32,
                    2,
                    gl::FLOAT,
                    0,
                    0,
                    position.as_ptr() as *const c_void,
                );
                gl::VertexAttribPointer(
                    color_location as u32,
                    4,
                    gl::FLOAT,
                    0,
                    0,
                    vert_color.as_ptr() as *const c_void,
                );
                gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4);
            },

            crate::Command::WriteString(x, y, width, height, color, text) => {}
        }
    }

    #[inline]
    fn end(&mut self) {
        self.wgl.swap_buffers();
    }
}
