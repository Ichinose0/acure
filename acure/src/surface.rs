use crate::{Context, Command};

pub trait Surface : Sized {
    fn width(&mut self,width: u32);
    fn height(&mut self,height: u32);
    fn command(&self,ctx: &[Command]);
}