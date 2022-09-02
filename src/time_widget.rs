use crossterm::style::Attribute;
use time::{
    OffsetDateTime,
};
use crate::{
    StatuslineWidget,
    StatuslineFormatter,
    Colors,
};



pub struct Time(String);
impl StatuslineWidget for Time {
    fn display(&self,f:&mut StatuslineFormatter) {
        let colors=Colors::default();
        f.write_section(&self.0,colors.background,colors.blue,vec![Attribute::Bold]);
    }
}
impl Time {
    pub fn new()->Time {
        let local=OffsetDateTime::now_local().unwrap();
        let content=format!("{:02}:{:02}:{:02}",local.hour(),local.minute(),local.second());
        Time(content)
    }
}
