use crate::{AlignMode, Command, LayoutMode, Color};

pub trait Surface: Sized {
    fn resize(&mut self,width: u32,height: u32);
    fn clear(&self,color: Color);
    fn command(&self, ctx: &[Command], align: AlignMode, layout: LayoutMode);
}
