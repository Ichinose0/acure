const FRAGMENT: &'static str = include_str!("shader/shader_core.frag");
const VERTEX: &'static str = include_str!("shader/shader_core.vert");

use std::{
    ffi::{c_void, CString},
    mem::{size_of, MaybeUninit},
    ptr::{null, null_mut},
};

use gl::types::{GLboolean, GLfloat, GLuint};
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::HWND,
        Graphics::{
            Gdi::{GetDC, HDC},
            OpenGL::*,
        },
    },
};

use crate::gl::{compile_shader, create_program, Vao, Vbo};

type wglCreateContextAttribsARB =
    fn(hDC: *mut c_void, hshareContext: *mut c_void, attribList: *const i32) -> *mut c_void;
type wglSwapIntervalEXT = fn(i32);

const WGL_CONTEXT_MAJOR_VERSION_ARB: i32 = 0x2091;
const WGL_CONTEXT_MINOR_VERSION_ARB: i32 = 0x2092;
const WGL_CONTEXT_LAYER_PLANE_ARB: i32 = 0x2093;
const WGL_CONTEXT_FLAGS_ARB: i32 = 0x2094;
const WGL_CONTEXT_PROFILE_MASK_ARB: i32 = 0x9126;

const WGL_CONTEXT_DEBUG_BIT_ARB: i32 = 0x0001;
const WGL_CONTEXT_FORWARD_COMPATIBLE_BIT_ARB: i32 = 0x0002;

const WGL_CONTEXT_CORE_PROFILE_BIT_ARB: i32 = 0x00000001;
const WGL_CONTEXT_COMPATIBILITY_PROFILE_BIT_ARB: i32 = 0x00000002;

const ERROR_INVALID_VERSION_ARB: i32 = 0x2095;
const ERROR_INVALID_PROFILE_ARB: i32 = 0x2096;

static VERTEX_DATA: [f32; 6] = [0.0, 0.5, 0.5, -0.5, -0.5, -0.5];

pub struct Wgl {
    wglCreateContextAttribsARB: wglCreateContextAttribsARB,
    wglSwapIntervalEXT: wglSwapIntervalEXT,
    hdc: HDC,
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
        let new_ctx = HGLRC((wglCreateContextAttribsARB)(
            hdc.0 as *mut c_void,
            null_mut(),
            attribs.as_ptr(),
        ) as isize);
        unsafe {
            wglDeleteContext(ctx);
            wglMakeCurrent(hdc, new_ctx);
        }

        let wglSwapIntervalEXT: wglSwapIntervalEXT =
            unsafe { std::mem::transmute(func("wglSwapIntervalEXT")) };

        Self {
            wglCreateContextAttribsARB,
            wglSwapIntervalEXT,
            hdc,
        }
    }

    #[inline]
    pub fn make_current(&self) {}

    #[inline]
    pub fn swap_intervals(&self, interval: bool) {
        ((self.wglSwapIntervalEXT)(interval as i32));
    }

    #[inline]
    pub fn swap_buffers(&self) {
        unsafe {
            SwapBuffers(self.hdc).unwrap();
        }
    }

    #[inline]
    pub fn get_proc_address(&self, procname: &str) -> *const c_void {
        unsafe {
            wglGetProcAddress(PCSTR(format!("{}\0", procname).as_ptr())).unwrap() as *const c_void
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
    projection: Vec<f32>,
    width: f32,
    height: f32,
}

impl WglSurface {
    #[inline]
    pub fn new(window: isize) -> Self {
        let hwnd = HWND(window);
        unsafe {
            let wgl = Wgl::load_with(hwnd, |s| {
                wglGetProcAddress(PCSTR(format!("{}\0", s).as_ptr())).unwrap() as *const c_void
            });

            gl::load_with(|s| wgl.get_proc_address(s));

            wgl.swap_intervals(true);

            let vertex = compile_shader(gl::VERTEX_SHADER, VERTEX);
            let fragment = compile_shader(gl::FRAGMENT_SHADER, FRAGMENT);

            let program = create_program(&[vertex, fragment]);

            let left = 0.0;
            let right = 1600.0;
            let bottom = 0.0;
            let top = 900.0;
            let near_val = -1.0;
            let far_val = 1.0;

            let projection: Vec<f32> = vec![
                2.0 / (right - left),
                0.0,
                0.0,
                0.0,
                0.0,
                2.0 / (top - bottom),
                0.0,
                0.0,
                0.0,
                0.0,
                -2.0 / (far_val - near_val),
                0.0,
                -(right + left) / (right - left),
                -(top + bottom) / (top - bottom),
                -(far_val + near_val) / (far_val - near_val),
                1.0,
            ];

            Self {
                hwnd,
                wgl,
                vertex,
                fragment,
                program,
                projection,
                width: 0.0,
                height: 0.0,
            }
        }
    }
}

impl crate::Surface for WglSurface {
    #[inline]
    fn surface_resize(&mut self, width: u32, height: u32) {
        self.width = width as f32;
        self.height = height as f32;
        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
            self.projection[0] = 2.0 / self.width;
            self.projection[5] = 2.0 / self.height;
            self.projection[12] = -(self.width + 0.0) / (self.width - 0.0);
            self.projection[13] = -(self.height + 0.0) / (self.height - 0.0);
        }
    }

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
                let vert_color: Vec<f32>;
                match color {
                    crate::Color::ARGB(a, r, g, b) => {
                        let a = (*a as f32) / 255.0;
                        let r = (*r as f32) / 255.0;
                        let g = (*g as f32) / 255.0;
                        let b = (*b as f32) / 255.0;
                        vert_color = vec![r, g, b, a];
                    }
                }

                let x = *x as f32;
                let y = *y as f32;
                let width = *width as f32;
                let height = *height as f32;

                let vao = Vao::new(1);
                let vbo = Vbo::gen(&[
                    x,
                    self.height-y,
                    vert_color[0],
                    vert_color[1],
                    vert_color[2],
                    vert_color[3],
                    x,
                    self.height-y-height,
                    vert_color[0],
                    vert_color[1],
                    vert_color[2],
                    vert_color[3],
                    x+width,
                    self.height-y-height,
                    vert_color[0],
                    vert_color[1],
                    vert_color[2],
                    vert_color[3],
                    x+width,
                    self.height-y,
                    vert_color[0],
                    vert_color[1],
                    vert_color[2],
                    vert_color[3],
                ]);

                gl::UseProgram(self.program);
                let matrix = gl::GetUniformLocation(
                    self.program,
                    CString::new("projectionMatrix").unwrap().as_ptr(),
                );
                gl::UniformMatrix4fv(matrix, 1, gl::FALSE as GLboolean, self.projection.as_ptr());
                let color_pos =
                    gl::GetAttribLocation(self.program, CString::new("color").unwrap().as_ptr());
                gl::EnableVertexAttribArray(color_pos as u32);
                gl::VertexAttribPointer(
                    color_pos as u32,
                    4,
                    gl::FLOAT,
                    gl::FALSE as GLboolean,
                    0,
                    vert_color.as_ptr() as *const c_void,
                );
                gl::BindFragDataLocation(
                    self.program,
                    0,
                    CString::new("out_color").unwrap().as_ptr(),
                );

                gl::VertexAttribPointer(
                    0,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    (6 * std::mem::size_of::<GLfloat>()) as i32,
                    null(),
                );
                gl::EnableVertexAttribArray(0);
                gl::VertexAttribPointer(
                    1,
                    4,
                    gl::FLOAT,
                    gl::FALSE,
                    (6 * std::mem::size_of::<GLfloat>()) as i32,
                    (3 * std::mem::size_of::<GLfloat>()) as *const c_void,
                );
                gl::EnableVertexAttribArray(1);

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
