use git2::{
    Repository,
    RepositoryOpenFlags,
    Branch,
};
use crossterm::style::Attribute;
use std::path::Path;
use crate::{
    StatuslineWidget,
    StatuslineFormatter,
    Colors,
};


pub struct Git(String);
impl StatuslineWidget for Git {
    fn display(&self,f:&mut StatuslineFormatter) {
        let colors=Colors::default();
        f.write_section(&self.0,colors.purple,colors.black,vec![Attribute::Bold]);
    }
}
impl Git {
    pub fn new<T:AsRef<Path>>(pwd:T)->Git {
        let content=if let Ok(repo)=Repository::open_ext::<_,&str,_>(pwd,RepositoryOpenFlags::empty(),[]) {
            if let Ok(head)=repo.head() {
                let commit=head.peel_to_commit().unwrap();
                let id_string=format!("{}",commit.id());
                let id_str=&id_string[..7];
                let branch=Branch::wrap(head);
                if let Ok(Some(branch_name))=branch.name() {
                    format!("Git{}{}",branch_name,id_str)
                } else {
                    format!("GitNULL{}",id_str)
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        Git(content)
    }
}
