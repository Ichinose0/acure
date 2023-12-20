#[macro_use]
extern crate log;

pub mod surface;

#[cfg(target_os = "windows")]
#[cfg(feature = "d2d1")]
pub mod d2d1;
#[cfg(target_os = "linux")]
pub mod x11;

use std::sync::Mutex;

use surface::Surface;


pub type AeResult<T> = Result<T, AcureResult>;

#[derive(Clone, Copy, Debug)]
pub enum AcureResult {
    UnauthorizedOperation,
}

#[derive(Clone, Copy, Debug)]
pub enum Color {
    ARGB(u8, u8, u8, u8),
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ContextState {
    Begin,
    End,
}

pub struct AcureBuilder {}

pub struct Acure {
    buffer: Vec<Command>,
    bgr: Color,
    align: AlignMode,
    layout: LayoutMode,
    state: ContextState,
    thickness: u32,
}

impl Acure {
    pub fn new() -> Self {
        Self {
            buffer: vec![],
            bgr: Color::ARGB(0, 0, 0, 0),
            align: AlignMode::Flex,
            layout: LayoutMode::NoCare,
            state: ContextState::End,
            thickness: 1,
        }
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.bgr = color;
    }

    pub fn set_align_mode(&mut self, mode: AlignMode) {
        self.align = mode;
    }

    pub fn set_layout_mode(&mut self, mode: LayoutMode) {
        self.layout = mode;
    }

    pub fn clear(&mut self) {
        if !self.buffer.is_empty() {
            self.buffer.clear();
        }
    }

    pub fn begin<T>(&mut self,surface: &mut T) 
    where
        T: Surface
    {
        self.state = ContextState::Begin;
        surface.begin();
    }

    pub fn push(&mut self, command: Command) {
        self.buffer.push(command);
    }

    pub fn write<T>(&mut self, surface: &mut T) -> AeResult<()>
    where
        T: Surface,
    {
        if self.state == ContextState::Begin {
            surface.clear(self.bgr);
            surface.command(&self.buffer, self.align, self.layout);
            surface.end();
            self.state = ContextState::End;
            return Ok(());
        }

        return Err(AcureResult::UnauthorizedOperation);
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}
