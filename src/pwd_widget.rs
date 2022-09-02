use crossterm::style::Attribute;
use crate::{
    StatuslineWidget,
    StatuslineFormatter,
    Colors,
};


pub struct Pwd(String);
impl StatuslineWidget for Pwd {
    fn display(&self,f:&mut StatuslineFormatter) {
        let colors=Colors::default();
        f.write_section(&self.0,colors.black,colors.green,vec![Attribute::Bold]);
    }
}
impl Pwd {
    pub fn new(wd:&str)->Pwd {
        Pwd(wd.to_string())
    }
}
