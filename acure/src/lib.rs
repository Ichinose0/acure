pub mod surface;

#[cfg(target_os = "windows")]
pub mod gdi;
#[cfg(target_os = "linux")]
pub mod x11;

use std::sync::Mutex;

use surface::Surface;

pub type Context = Mutex<Vec<Command>>;

pub enum Color {
    ARGB(u8,u8,u8,u8)
}

pub enum Command {
    Clear(Color)
}

pub struct Acure {
    ctx: Context
}

impl Acure {
    pub fn new() -> Self {
        Self {
            ctx: Mutex::new(vec![])
        }
    }

    pub fn push(&self,command: Command) {
        self.ctx.lock().unwrap().push(command);
    }

    pub fn write<T>(surface: T)
    where
        T: Surface
    {
        
    }
}