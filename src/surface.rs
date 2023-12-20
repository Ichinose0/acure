use crate::{AlignMode, Color, Command, LayoutMode};

pub trait Surface: Sized {
    fn surface_resize(&mut self, width: u32, height: u32);

    fn begin(&self);
    fn clear(&self, color: Color);
    fn command(&self, ctx: &[Command], align: AlignMode, layout: LayoutMode);
    fn end(&self);
}
