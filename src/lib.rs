pub mod surface;

#[cfg(target_os = "windows")]
#[cfg(feature = "d2d1")]
pub mod d2d1;
#[cfg(target_os = "linux")]
pub mod x11;

use std::sync::Mutex;

use surface::Surface;

#[derive(Clone, Copy, Debug)]
pub enum Color {
    ARGB(u8, u8, u8, u8),
}

#[derive(Clone,Debug)]
pub enum Command {
    // X,Y,Width,Height,Radius,Color
    FillRectangle(u32, u32, u32, u32, f64, Color),
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
    buffer: Vec<Command>>,
    bgr: Color,
    align: AlignMode,
    layout: LayoutMode,
    thickness: u32,
}

impl<'a> Acure<'a> {
    pub fn new() -> Self {
        Self {
            buffer: vec![],
            color: Color::ARGB(0,0,0,0),
            align: AlignMode::Flex,
            layout: LayoutMode::NoCare,
            thickness: 1,
        }
    }

    pub fn set_align_mode(&mut self, mode: AlignMode) {
        self.align = mode;
    }

    pub fn set_layout_mode(&mut self, mode: LayoutMode) {
        self.layout = mode;
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn push(&mut self, command: Command) {
        self.buffer.push(command);
    }

    pub fn write<T>(&self, surface: &mut T)
    where
        T: Surface,
    {
        let mut bgr = self.bgr;
        let mut cmds = vec![];
        for c in self.buffer {

        }
        surface.command(
            &self.ctx.lock().unwrap(),
            *self.align.lock().unwrap(),
            *self.layout.lock().unwrap(),
        );
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}
