use crate::Command;
use crate::surface::Surface;

pub struct GDISurface {
    width: u32,
    height: u32
}

impl Surface for GDISurface {
    fn width(&mut self,width: u32) {
        self.width = width;
    }

    fn height(&mut self,height: u32) {
        self.height = height;
    }

    fn command(&self,ctx: &[Command]) {

    }
}