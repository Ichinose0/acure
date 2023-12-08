use winapi::shared::d3d9types::D3DCOLORVALUE;
use winapi::shared::dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM;
use winapi::shared::windef::{HWND, RECT};
use winapi::shared::winerror::D2DERR_RECREATE_TARGET;
use winapi::um::d2d1::{
    D2D1CreateFactory, ID2D1Brush, ID2D1Factory, D2D1_BRUSH_PROPERTIES,
    D2D1_DRAW_TEXT_OPTIONS_CLIP, D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_FEATURE_LEVEL,
    D2D1_FEATURE_LEVEL_DEFAULT, D2D1_HWND_RENDER_TARGET_PROPERTIES, D2D1_PRESENT_OPTIONS_NONE,
    D2D1_RECT_F, D2D1_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_TYPE_DEFAULT,
    D2D1_RENDER_TARGET_USAGE, D2D1_RENDER_TARGET_USAGE_NONE, D2D1_ROUNDED_RECT, D2D1_SIZE_U,
};
use winapi::um::dcommon::{
    D2D1_ALPHA_MODE, D2D1_ALPHA_MODE_IGNORE, D2D_MATRIX_3X2_F, DWRITE_MEASURING_MODE_NATURAL,
};
use winapi::um::dwrite::{
    DWriteCreateFactory, IDWriteFactory, DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_STRETCH_NORMAL,
    DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_REGULAR, DWRITE_PARAGRAPH_ALIGNMENT_CENTER,
    DWRITE_TEXT_ALIGNMENT_CENTER,
};
use winapi::um::winuser::{GetClientRect, GetDpiForWindow};
use winapi::Interface;

use std::ptr::{null, null_mut};

use crate::surface::Surface;
use crate::{AlignMode, Color, Command, LayoutMode};

#[macro_use]
macro_rules! SafeRelease {
    ($p:expr) => {
        unsafe {
            (*$p).Release();
        }
    };
}

pub struct D2D1Surface {
    hwnd: isize,
    factory: *mut ID2D1Factory,
    dwrite_factory: *mut IDWriteFactory,
    width: u32,
    height: u32,
}

impl D2D1Surface {
    pub fn new(hwnd: isize) -> Self {
        let mut factory = unsafe { std::mem::zeroed() };
        let mut dwrite_factory = unsafe { std::mem::zeroed() };
        let riid = ID2D1Factory::uuidof();
        unsafe {
            D2D1CreateFactory(
                D2D1_FACTORY_TYPE_SINGLE_THREADED,
                &riid,
                null(),
                &mut factory,
            );
            let riid = IDWriteFactory::uuidof();
            DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED, &riid, &mut dwrite_factory);
        }

        Self {
            hwnd,
            factory: factory as *mut ID2D1Factory,
            dwrite_factory: dwrite_factory as *mut IDWriteFactory,
            width: 1,
            height: 1,
        }
    }
}

impl Surface for D2D1Surface {
    fn width(&mut self, width: u32) {
        self.width = width;
    }

    fn height(&mut self, height: u32) {
        self.height = height;
    }

    fn command(&self, ctx: &[Command], align: AlignMode, layout: LayoutMode) {
        let mut hr = 0;
        let mut render_target;
        unsafe {
            let render_props = D2D1_RENDER_TARGET_PROPERTIES {
                _type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
                pixelFormat: winapi::um::dcommon::D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_IGNORE,
                },
                dpiX: 96.0,
                dpiY: 96.0,
                usage: D2D1_RENDER_TARGET_USAGE_NONE,
                minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
            };
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            GetClientRect(isize::from(self.hwnd) as HWND, &mut rect);
            let size = D2D1_SIZE_U {
                width: rect.right as u32,
                height: rect.bottom as u32,
            };
            let hwnd_props = D2D1_HWND_RENDER_TARGET_PROPERTIES {
                hwnd: isize::from(self.hwnd) as HWND,
                pixelSize: size,
                presentOptions: D2D1_PRESENT_OPTIONS_NONE,
            };
            render_target = std::mem::zeroed();
            hr = (*self.factory).CreateHwndRenderTarget(
                &render_props,
                &hwnd_props,
                &mut render_target,
            );
        }

        unsafe {
            let matrix = D2D_MATRIX_3X2_F {
                matrix: [[1.0, 0.0], [0.0, 1.0], [0.0, 0.0]],
            };
            let dpi = GetDpiForWindow(self.hwnd as HWND);
            (*render_target).SetDpi(dpi as f32, dpi as f32);
            (*render_target).BeginDraw();
            (*render_target).SetTransform(&matrix);
        }
        for c in ctx {
            match c {
                Command::Clear(color) => {
                    let color = create_d3dcolorvalue(*color);
                    unsafe {
                        (*render_target).Clear(&color);
                    }
                }
                Command::FillRectangle(x, y, width, height, radius, color) => {
                    let color = create_d3dcolorvalue(*color);
                    let mut brush = unsafe { std::mem::zeroed() };
                    unsafe { (*render_target).CreateSolidColorBrush(&color, null(), &mut brush) };

                    let rect = D2D1_RECT_F {
                        left: *x as f32,
                        top: *y as f32,
                        right: (*x + (*width)) as f32,
                        bottom: (*y + (*height)) as f32,
                    };
                    if *radius == 0.0 {
                        unsafe {
                            (*render_target).FillRectangle(&rect, brush as *mut ID2D1Brush);
                        }
                    } else {
                        let rounded_rect = D2D1_ROUNDED_RECT {
                            rect: rect,
                            radiusX: *radius as f32,
                            radiusY: *radius as f32,
                        };
                        unsafe {
                            (*render_target)
                                .FillRoundedRectangle(&rounded_rect, brush as *mut ID2D1Brush);
                        }
                    }
                    SafeRelease!(brush);
                }
                Command::WriteString(x, y, width, height, color, string) => {
                    let color = create_d3dcolorvalue(*color);
                    let mut string = string.encode_utf16().collect::<Vec<u16>>();
                    string.push(0);
                    let mut font_name = "Yu gothic".encode_utf16().collect::<Vec<u16>>();
                    font_name.push(0);
                    let mut lang = "en-us".encode_utf16().collect::<Vec<u16>>();
                    lang.push(0);
                    let mut text_format = unsafe { std::mem::zeroed() };
                    let font_size = (*height as f32) / 2.0;
                    unsafe {
                        (*self.dwrite_factory).CreateTextFormat(
                            font_name.as_ptr(),
                            null_mut(),
                            DWRITE_FONT_WEIGHT_REGULAR,
                            DWRITE_FONT_STYLE_NORMAL,
                            DWRITE_FONT_STRETCH_NORMAL,
                            font_size,
                            lang.as_ptr(),
                            &mut text_format,
                        );
                        (*text_format).SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
                        (*text_format).SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);
                    }
                    let mut brush = unsafe { std::mem::zeroed() };
                    unsafe { (*render_target).CreateSolidColorBrush(&color, null(), &mut brush) };

                    let layout_rect = D2D1_RECT_F {
                        left: *x as f32,
                        top: *y as f32,
                        right: (*x + (*width)) as f32,
                        bottom: (*y + (*height)) as f32,
                    };

                    unsafe {
                        (*render_target).DrawText(
                            string.as_ptr(),
                            string.len() as u32,
                            text_format,
                            &layout_rect,
                            brush as *mut ID2D1Brush,
                            D2D1_DRAW_TEXT_OPTIONS_CLIP,
                            DWRITE_MEASURING_MODE_NATURAL,
                        );
                        SafeRelease!(brush);
                        SafeRelease!(text_format);
                    }
                }
            }
        }
        let mut tag1 = 0;
        let mut tag2 = 0;
        unsafe {
            (*render_target).EndDraw(&mut tag1, &mut tag2);
            SafeRelease!(render_target);
        }
    }
}

fn create_d3dcolorvalue(color: Color) -> D3DCOLORVALUE {
    match color {
        Color::ARGB(a, r, g, b) => D3DCOLORVALUE {
            r: (r as f32) / 255.0,
            g: (g as f32) / 255.0,
            b: (b as f32) / 255.0,
            a: (a as f32) / 255.0,
        },
    }
}
