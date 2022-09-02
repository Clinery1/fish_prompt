use crossterm::style::Attribute;
use crate::{
    StatuslineWidget,
    StatuslineFormatter,
    Colors,
};


pub struct PipeStatus(String);
impl StatuslineWidget for PipeStatus {
    fn display(&self,f:&mut StatuslineFormatter) {
        if self.is_active() {
            let colors=Colors::default();
            f.write_section(&self.0,colors.black,colors.dark_red,vec![Attribute::Bold]);
        }
    }
}
impl PipeStatus {
    pub fn new(status_raw:&str)->PipeStatus {
        let mut pipe_status=String::new();
        let mut error=false;
        for status in status_raw.split(' ') {
            if status!="0" {error=true}
            pipe_status.push_str(status);
            pipe_status.push('ﳣ');
        }
        pipe_status.pop();
        let content=if error {
            pipe_status
        } else {
            String::new()
        };
        PipeStatus(content)
    }
    fn is_active(&self)->bool {
        !self.0.is_empty()
    }
}
