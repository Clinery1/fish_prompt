use crossterm::style::Attribute;
use crate::{
    StatuslineWidget,
    StatuslineFormatter,
    Colors,
};


pub struct FileCount(String);
impl StatuslineWidget for FileCount {
    fn display(&self,f:&mut StatuslineFormatter) {
        let colors=Colors::default();
        f.write_section(&self.0,colors.black,colors.yellow,vec![Attribute::Bold]);
    }
}
impl FileCount {
    pub fn new(count:usize)->FileCount {
        let content=if count==0 {
            format!("No files")
        } else if count==1 {
            format!("1 file")
        } else {
            format!("{} files",count)
        };
        FileCount(content)
    }
}
