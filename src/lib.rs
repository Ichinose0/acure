pub mod surface;

#[cfg(target_os = "windows")]
#[cfg(feature = "gdi")]
pub mod gdi;
#[cfg(target_os = "windows")]
#[cfg(feature = "d2d1")]
pub mod d2d1;
#[cfg(target_os = "linux")]
pub mod x11;

use std::sync::Mutex;

use surface::Surface;

pub type Context = Mutex<Vec<Command>>;

#[derive(Clone, Copy, Debug)]
pub enum Color {
    ARGB(u8, u8, u8, u8),
}

pub enum Command {
    Clear(Color),
    // X,Y,Width,Height,Color
    FillRectangle(u32, u32, u32, u32, Color),
    WriteString(u32, u32, u32, u32, Color, String),
}

#[derive(Clone, Copy, Debug)]
pub enum LayoutMode {
    NoCare,
    AdjustSize,
}

#[derive(Clone, Copy, Debug)]
pub enum AlignMode {
    CenterAligned,
    RightAligned,
    LeftAligned,
    TopAligned,
    BottomAligned,
    Flex,
}

pub struct Acure {
    ctx: Context,
    align: Mutex<AlignMode>,
    layout: Mutex<LayoutMode>,
    thickness: u32,
}

impl Acure {
    pub fn new() -> Self {
        Self {
            ctx: Mutex::new(vec![]),
            align: Mutex::new(AlignMode::Flex),
            layout: Mutex::new(LayoutMode::NoCare),
            thickness: 1,
        }
    }

    pub fn set_align_mode(&self, mode: AlignMode) {
        *self.align.lock().unwrap() = mode;
    }

    pub fn set_layout_mode(&self, mode: LayoutMode) {
        *self.layout.lock().unwrap() = mode;
    }

    pub fn clear(&self) {
        self.ctx.lock().unwrap().clear();
    }

    pub fn push(&self, command: Command) {
        self.ctx.lock().unwrap().push(command);
    }

    pub fn write<T>(&self, surface: &T)
    where
        T: Surface,
    {
        surface.command(
            &self.ctx.lock().unwrap(),
            *self.align.lock().unwrap(),
            *self.layout.lock().unwrap(),
        );
    }
}
