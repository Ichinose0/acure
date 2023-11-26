use std::ffi::c_void;
use std::ptr::null_mut;

use gdiplus_sys2::{
    GdipDeleteBrush, GdipDeleteGraphics, GdipFillRectangleI, GdiplusStartup, GdiplusStartupInput,
    GpBrush, Status_Ok, HWND,
};
use winapi::um::wingdi::{SetBkMode, TRANSPARENT, SetTextColor, CreateFontW, SelectObject, TextOutW, DeleteObject, RGB, SHIFTJIS_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, FF_MODERN, VARIABLE_PITCH, DEFAULT_QUALITY};

use crate::surface::Surface;
use crate::{Color, Command};

const FALSE: i32 = 0;

pub struct GDISurface {
    hwnd: isize,
    token: usize,
    input: GdiplusStartupInput,
    width: u32,
    height: u32,
}

impl GDISurface {
    pub fn new(hwnd: isize) -> Self {
        let mut token = 0;
        let input = GdiplusStartupInput {
            GdiplusVersion: 1,
            DebugEventCallback: None,
            SuppressBackgroundThread: FALSE,
            SuppressExternalCodecs: FALSE,
        };
        unsafe {
            let status = GdiplusStartup(&mut token, &input, null_mut());
            println!("{}", status);
            if status != Status_Ok {
                panic!("Can't startup GDI+");
            }
        }
        Self {
            hwnd,
            token,
            input,
            width: 1,
            height: 1,
        }
    }
}

impl Surface for GDISurface {
    fn width(&mut self, width: u32) {
        self.width = width;
    }

    fn height(&mut self, height: u32) {
        self.height = height;
    }

    fn command(&self, ctx: &[Command]) {
        let mut ps: winapi::um::winuser::PAINTSTRUCT;
        let hdc;
        let mut graphics = null_mut();
        unsafe {
            ps = std::mem::zeroed();
            hdc = winapi::um::winuser::BeginPaint(self.hwnd as HWND, &mut ps);
            let status = gdiplus_sys2::GdipCreateFromHDC(hdc, &mut graphics);
        }
        for c in ctx {
            match c {
                Command::Clear(color) => {
                    let mut brush = null_mut();
                    unsafe {
                        gdiplus_sys2::GdipCreateSolidFill(color_to_argb(*color), &mut brush);
                        GdipFillRectangleI(
                            graphics,
                            brush as *mut GpBrush,
                            ps.rcPaint.left as i32,
                            ps.rcPaint.top as i32,
                            (ps.rcPaint.right - ps.rcPaint.left) as i32,
                            (ps.rcPaint.bottom - ps.rcPaint.top) as i32,
                        );
                        GdipDeleteBrush(brush as *mut GpBrush);
                    }
                }
                Command::FillRectangle(x, y, width, height, color) => {
                    let mut brush = null_mut();
                    unsafe {
                        gdiplus_sys2::GdipCreateSolidFill(color_to_argb(*color), &mut brush);
                        GdipFillRectangleI(
                            graphics,
                            brush as *mut GpBrush,
                            *x as i32,
                            *y as i32,
                            *width as i32,
                            *height as i32,
                        );
                        GdipDeleteBrush(brush as *mut GpBrush);
                    }
                }
                Command::WriteString(x, y, width, height, color, string) => {
                    let mut v: Vec<u16> = string.encode_utf16().collect();
                    v.push(0);
                    let f = "Elite";
                    let mut font_name: Vec<u16> = f.encode_utf16().collect();
                    font_name.push(0);
                    unsafe {
                        SetBkMode(hdc, TRANSPARENT as i32);
                        SetTextColor(hdc, color_to_rgb(*color));

                        let font = CreateFontW(*height as i32,0,0,0,0,0,0,0,SHIFTJIS_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, DEFAULT_QUALITY,VARIABLE_PITCH | FF_MODERN, font_name.as_ptr());

                        SelectObject(hdc, font as *mut c_void);
                        TextOutW(hdc,*x as i32, *y as i32,v.as_ptr(),v.len() as i32);
                        DeleteObject(font as *mut c_void);
                    }
                }
            }
        }
        unsafe {
            GdipDeleteGraphics(graphics);
            winapi::um::winuser::EndPaint(self.hwnd as HWND, &ps);
        }
    }
}

fn argb(alpha: u8, red: u8, green: u8, blue: u8) -> u32 {
    ((alpha as u32) << 24) | ((red as u32) << 16) | ((green as u32) << 8) | (blue as u32)
}

fn color_to_argb(color: Color) -> u32 {
    match color {
        Color::ARGB(a, r, g, b) => argb(a, r, g, b),
    }
}

fn color_to_rgb(color: Color) -> u32 {
    match color {
        Color::ARGB(a, r, g, b) => RGB(r, g, b),
    }
}
