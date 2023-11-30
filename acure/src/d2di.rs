use winapi::um::d2d1::{D2D1CreateFactory,ID2D1Factory,D2D1_FACTORY_TYPE_SINGLE_THREADED};
use winapi::um::dwrite::{DWriteCreateFactory,DWRITE_FACTORY_TYPE_SHARED};

use std::ptr::{null,null_mut};

use crate::surface::Surface;
use crate::{AlignMode, Color, Command, LayoutMode};

pub struct D2D1Surface {
    hwnd: isize,
    width: u32,
    height: u32,
}

impl D2D1Surface {
    pub fn new(hwnd: isize) -> Self {
        let mut factory = unsafe { std::mem::zeroed() };
        let riid = ID2D1Factory.uuidof();
        unsafe {
            D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED,&riid,null(),&mut factory);
            DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED);
        }
        Self {
            hwnd,
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
        for c in ctx {
            debug!("Scheduled drawing commands: {:#?}",c);
            match c {
                Command::Clear(color) => {

                }
                Command::FillRectangle(x, y, width, height, color) => {

                }
                Command::WriteString(x, y, width, height, color, string) => {

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
