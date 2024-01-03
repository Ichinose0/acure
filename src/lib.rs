#[macro_use]
extern crate log;

pub mod surface;

#[cfg(target_os = "windows")]
#[cfg(feature = "d2d1")]
pub mod d2d1;
#[cfg(target_os = "linux")]
#[cfg(feature = "x11")]
pub mod x11;
#[cfg(target_os = "linux")]
#[cfg(feature = "x11_egl")]
pub mod x11egl;

#[cfg(target_os = "windows")]
#[cfg(feature = "wgl")]
pub mod wgl;

#[cfg(feature = "gl")]
pub(crate) mod gl;

use std::{fmt::Display, sync::Mutex};

use surface::Surface;
use thiserror::Error;

pub type AeResult<T> = Result<T, AcureError>;

#[derive(Debug)]
pub enum Backend {
    D2D1,
    WGL,
    X11EGL,
}

impl Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum AcureError {
    #[error("This operation is not authorized.")]
    UnauthorizedOperation,
    #[error("Null pointer detected.\n'{0}'")]
    NullPtrError(String),
    #[error("Backend: '{0}'\n'{1}'")]
    BackendError(Backend, anyhow::Error),
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
    #[inline]
    pub const fn new() -> Self {
        Self {
            buffer: vec![],
            bgr: Color::ARGB(0, 0, 0, 0),
            align: AlignMode::Flex,
            layout: LayoutMode::NoCare,
            state: ContextState::End,
            thickness: 1,
        }
    }

    #[inline]
    pub fn set_background_color(&mut self, color: Color) {
        self.bgr = color;
    }

    #[inline]
    pub fn set_align_mode(&mut self, mode: AlignMode) {
        self.align = mode;
    }

    #[inline]
    pub fn set_layout_mode(&mut self, mode: LayoutMode) {
        self.layout = mode;
    }

    #[inline]
    pub fn clear(&mut self) {
        if !self.buffer.is_empty() {
            self.buffer.clear();
        }
    }

    #[inline]
    pub fn begin<T>(&mut self, surface: &mut T)
    where
        T: Surface,
    {
        self.state = ContextState::Begin;
        surface.begin();
    }

    #[inline]
    pub fn push(&mut self, command: Command) {
        self.buffer.push(command);
    }

    #[inline]
    pub fn write<T>(&mut self, surface: &mut T) -> AeResult<()>
    where
        T: Surface,
    {
        if self.state == ContextState::Begin {
            surface.clear(self.bgr);
            for i in &self.buffer {
                surface.command(i, self.align, self.layout);
            }

            surface.end();
            self.state = ContextState::End;
            return Ok(());
        }

        return Err(AcureError::UnauthorizedOperation);
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}
