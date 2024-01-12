use crate::{AlignMode, Color, Command, LayoutMode};

pub trait Surface {
    fn surface_resize(&mut self, width: u32, height: u32);

    fn begin(&mut self);
    fn clear(&self, color: Color);
    fn command(&self, command: &Command, align: AlignMode, layout: LayoutMode);
    fn end(&mut self);
}
