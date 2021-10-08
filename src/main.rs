use git2::{
    Repository,
    Branch,
};
use termion::{
    style::{
        Bold,
        Reset as SReset,
    },
    color::{
        Rgb,
        Reset,
    },
    clear::{
        CurrentLine as ClearCurrentLine,
    },
    cursor::Up as CursorUp,
};
use chrono::prelude::*;
use std::{
    fmt::{
        self,
        Display,
        Formatter,
    },
    fs::{
        read_dir,
        FileType,
        read_link,
    },
    os::unix::fs::{
        FileTypeExt,
        MetadataExt,
    },
    env::{
        vars,
        args,
    },
    path::Path,
    collections::HashMap,
    process::Command,
    io,
};


#[derive(PartialEq)]
enum ListFileType {
    File {
        name:String,
        primary:FileIcon,
        secondary:Option<FileIcon>,
        executable:bool,
    },
    Directory {
        name:String,
        icon:DirIcon,
        items:usize,
    },
    Symlink {
        name:String,
        icon:SymlinkIcon,
    },
    Character {
        name:String,
    },
    Block {
        name:String,
    },
}
impl ListFileType {
    fn new_dir(name:String,pwd:String,items:usize)->ListFileType {
        let mut icon=DirIcon::from(name.as_str());
        if icon==DirIcon::ClosedFolder&&items>0 {
            icon=DirIcon::OpenFolder;
        }
        if pwd=="/home" {
            icon=DirIcon::HomeFolder;
        }
        ListFileType::Directory {
            name,
            icon,
            items,
        }
    }
    fn new_symlink(name:String,file_type:FileType)->ListFileType {
        let icon;
        if file_type.is_dir() {
            icon=SymlinkIcon::Dir;
        } else if file_type.is_symlink() {
            icon=SymlinkIcon::Link;
        } else if file_type.is_char_device() {
            icon=SymlinkIcon::Char;
        } else if file_type.is_block_device() {
            icon=SymlinkIcon::Block;
        } else {
            icon=SymlinkIcon::File;
        }
        ListFileType::Symlink {
            name,
            icon,
        }
    }
    fn new_char(name:String)->ListFileType {
        ListFileType::Character{name}
    }
    fn new_block(name:String)->ListFileType {
        ListFileType::Block{name}
    }
    fn new_file(name:String,executable:bool)->ListFileType {
        let (primary,secondary)=FileIcon::from_str(name.as_str());
        ListFileType::File {
            primary,
            secondary,
            executable,
            name,
        }
    }
}
impl Display for ListFileType {
    fn fmt(&self,formatter:&mut Formatter)->fmt::Result {
        use ListFileType::*;
        if formatter.alternate() {  // uncolored
            match self {
                File{name,primary,secondary,..}=>{  // we dont care about executable cause we have no colors
                    let s;
                    if let Some(secondary)=secondary {
                        s=format!("{}",secondary);
                    } else {
                        s=String::new();
                    }
                    write!(formatter," {}{}{}",primary,s,name)
                },
                Directory{name,icon,items}=>{
                    let i=if *items>0 {
                        format!("[{}]",items)
                    } else {String::new()};
                    write!(formatter," {}{}{}",icon,name,i)
                },
                Symlink{name,icon}=>write!(formatter," {}{}",icon,name),
                Character{name}=>write!(formatter," {}",name),
                Block{name}=>write!(formatter," {}",name),
            }
        } else {
            let black=Rgb(0x0,0x0,0x0);
            let dark_blue=Rgb(0x34,0x65,0xA4);
            let cyan=Rgb(0x06,0x98,0x9A);
            let device_color=Rgb(0xC4,0xA0,0x00);
            let executable_color=Rgb(0x4E,0x9A,0x06);
            match self {
                File{name,primary,secondary,executable}=>{
                    let s;
                    if let Some(secondary)=secondary {
                        s=format!("{}",secondary);
                    } else {
                        s=String::new();
                    }
                    let start_color=black.bg_string();
                    let mid_color=format!("{}{}",if *executable{executable_color.fg_string()}else{Reset.fg_str().to_string()},black.bg_string());
                    let end_color=format!("{}{}",black.fg_string(),Reset.bg_str());
                    write!(formatter,"{}{} {}{}{}{}",start_color,mid_color,primary,s,name,end_color)
                },
                Directory{name,icon,items}=>{
                    let i=if *items>0 {
                        format!("[{}]",items)
                    } else {String::new()};
                    let start_color=dark_blue.bg_string();
                    let mid_color=format!("{}{}",dark_blue.bg_string(),Reset.fg_str());
                    let end_color=format!("{}{}",Reset.bg_str(),dark_blue.fg_string());
                    write!(formatter,"{}{} {}{}{}{}",start_color,mid_color,icon,name,i,end_color)
                },
                Symlink{name,icon}=>{
                    let start_color=cyan.bg_string();
                    let mid_color=format!("{}{}",cyan.fg_string(),black.bg_string());
                    let end_color=format!("{}{}",Reset.bg_str(),cyan.fg_string());
                    write!(formatter,"{}{} {}{}{}",start_color,mid_color,icon,name,end_color)
                },
                Character{name}=>{
                    let start_color=black.bg_string();
                    let mid_color=format!("{}{}",device_color.fg_string(),black.bg_string());
                    let end_color=format!("{}{}",Reset.bg_str(),black.fg_string());
                    write!(formatter,"{}{} {}{}",start_color,mid_color,name,end_color)
                },
                Block{name}=>{
                    let start_color=black.bg_string();
                    let mid_color=format!("{}{}",device_color.fg_string(),black.bg_string());
                    let end_color=format!("{}{}",Reset.bg_str(),black.fg_string());
                    write!(formatter,"{}{} {}{}",start_color,mid_color,name,end_color)
                },
            }
        }
    }
}
/// Added after the link icon. Example: `` for a link to a folder
#[derive(PartialEq)]
enum SymlinkIcon {
    Link,
    Dir,
    File,
    Block,
    Char,
}
impl Display for SymlinkIcon {
    fn fmt(&self,formatter:&mut Formatter)->fmt::Result {
        use SymlinkIcon::*;
        match self {
            Link=>write!(formatter,""),
            Dir=>write!(formatter,""),
            File=>write!(formatter,""),
            Block=>write!(formatter,""),
            Char=>write!(formatter,""),
        }
    }
}
#[derive(PartialEq)]
enum DirIcon {
    Git,
    Pictures,
    Videos,
    Documents,
    Downloads,
    HomeFolder,
    Desktop,
    Sounds,
    CodeSource,
    Vim,
    OpenFolder,
    ClosedFolder,
}
impl From<&str> for DirIcon {
    fn from(name:&str)->DirIcon {
        use DirIcon::*;
        match name {
            ".git"=>Git,
            "Pictures"=>Pictures,
            "Videos"=>Videos,
            "Documents"=>Documents,
            "Downloads"=>Downloads,
            "Desktop"=>Desktop,
            "Sounds"=>Sounds,
            "src"=>CodeSource,
            ".vim"|".nvim"|"vim"|"nvim"|".neovim"|"neovim"=>Vim,
            _=>ClosedFolder,
        }
    }
}
impl Display for DirIcon {
    fn fmt(&self,formatter:&mut Formatter)->fmt::Result {
        use DirIcon::*;
        match self {
            Git=>write!(formatter,""),
            Pictures=>write!(formatter,""),
            Videos=>write!(formatter,""),
            Documents=>write!(formatter,""),
            Downloads=>write!(formatter,""),
            HomeFolder=>write!(formatter,""),
            Desktop=>write!(formatter,""),
            Sounds=>write!(formatter,""),
            CodeSource=>write!(formatter,""),
            Vim=>write!(formatter,""),
            OpenFolder=>write!(formatter,""),
            ClosedFolder=>write!(formatter,""),
        }
    }
}
#[derive(PartialEq)]
enum FileIcon {
    /// `Makefile`, `configure`, `build`, `*.ld`, `*.ninja`. Subject to change
    BuildTools,
    /// `LICENCE` filename
    License,
    /// `run` filename
    Run,
    /// `*.gdb`
    Gdb,
    /// `*.fish`
    Fish,
    /// `*.lock`
    Lock,
    /// `*.key`
    Key,
    /// `*.keys`
    Keys,
    /// `*.sh`
    Shell,
    /// `*.png`, `*.jpg`, `*.jpeg`
    Image,
    /// `*txt`, `*.odt`, `*.log`
    Text,
    /// `*.img`, `*.iso`, `*.vhd`
    HardDriveImage,
    /// `*.md`
    Markdown,
    /// `*.rs`
    Rust,
    /// `*.c`, `*.h`
    C,
    /// `*.cpp`, `*.hpp`, `*.ino` for arduino
    Cpp,
    /// `*.js`, `*.ts`
    Javascript,
    /// `*.py`
    Python,
    /// `*.html`
    Html,
    /// `*.css`
    Css,
    /// `*.exe`, `*.dll`
    Windows,
    /// `*.gz`, `*.zstd`, `*.xz`
    Compressed,
    /// `*.tar` `*.tar.FORMAT` where FORMAT is a compression format is stored as
    /// primary:Compressed, secondary:Some(Archive),
    Archive,
    /// `*.zip`, `*.rar`
    CompressedArchive,
    /// `*.apk`
    AndroidApp,
    /// `*.docx`, `*.odf`, `*.doc`
    Document,
    /// `*.xlsx`, `*.xls`
    Spreadsheet,
    /// `*.pptx`, `*.ppt`
    Presentation,
    /// `*.gitignore`
    Git,
    /// `*.vim`, `*.nvim`
    Vim,
    /// `*.ttf`, `*.psf`
    Font,
    /// `*.bin` `*.rom`
    Binary,
    /// `*.calc`
    Calc,
    /// `*.godot`, `*.gd`, `*.tscn`, `*.tsc`, `*.tres`
    Godot,
    /// `*.cfg`, `*.conf`, `*.toml`, `*.yml`, `*.json`, `*.ini`
    Config,
    /// `*history` so everything that ends with history
    History,
    /// Everything else
    Regular,
}
impl FileIcon {
    fn from_str(name:&str)->(FileIcon,Option<FileIcon>) {
        if name.contains('.') {
            let mut split=name.split('.').collect::<Vec<_>>();
            if split.len()>=2 {
                let primary=Self::match_extension(split.pop().unwrap());
                let mut secondary=None;
                if split.len()>=2 {
                    let s=Self::match_extension(split.pop().unwrap());
                    if s!=FileIcon::Regular {secondary=Some(s)}
                }
                return (primary,secondary);
            }
        }
        if name=="Makefile"||name=="build"||name=="configure" {return (FileIcon::BuildTools,None)}
        if name=="run" {return (FileIcon::Run,None)}
        if name=="LICENSE" {return (FileIcon::License,None)}
        return (FileIcon::Regular,None);
    }
    fn match_extension(ext:&str)->FileIcon {
        use FileIcon::*;
        match ext {
            "gitignore"=>Git,
            "apk"=>AndroidApp,
            "cfg"|"conf"|"toml"|"yml"|"json"|"ini"=>Config,
            "calc"=>Calc,
            "godot"|"gd"|"tscn"|"tsc"|"tres"=>Godot,
            "bin"|"rom"=>Binary,
            "psf"|"ttf"=>Font,
            "pptx"|"ppt"=>Presentation,
            "xlsx"|"xls"=>Spreadsheet,
            "docx"|"odf"|"doc"=>Document,
            "ld"|"ninja"=>BuildTools,
            "gdb"=>Gdb,
            "fish"=>Fish,
            "lock"=>Lock,
            "key"=>Key,
            "keys"=>Keys,
            "sh"=>Shell,
            "png"|"jpg"|"jpeg"=>Image,
            "txt"|"odt"|"log"=>Text,
            "img"|"iso"|"vhd"=>HardDriveImage,
            "md"=>Markdown,
            "rs"=>Rust,
            "c"|"h"=>C,
            "cpp"|"hpp"=>Cpp,
            ".vim"|".vimrc"|".nvimrc"|".nvim"=>Vim,
            "js"|"ts"=>Javascript,
            "py"=>Python,
            "html"=>Html,
            "css"=>Css,
            "exe"|"dll"=>Windows,
            "gz"|"zstd"|"xz"=>Compressed,
            "tar"=>Archive,
            "zip"|"rar"=>CompressedArchive,
            _=>{
                if ext.ends_with("history") {return History}
                return Regular;
            },
        }
    }
}
impl Display for FileIcon {
    fn fmt(&self,formatter:&mut Formatter)->fmt::Result {
        use FileIcon::*;
        match self {
            BuildTools=>write!(formatter,""),
            License=>write!(formatter,""),
            Run=>write!(formatter,"ﰌ"),
            Gdb=>write!(formatter,""),
            Fish=>write!(formatter,""),
            Lock=>write!(formatter,""),
            Key=>write!(formatter,""),
            Keys=>write!(formatter,""),
            Shell=>write!(formatter,""),
            Image=>write!(formatter,""),
            Text=>write!(formatter,""),
            HardDriveImage=>write!(formatter,""),
            Markdown=>write!(formatter,""),
            Rust=>write!(formatter,""),
            C=>write!(formatter,""),
            Cpp=>write!(formatter,""),
            Javascript=>write!(formatter,""),
            Python=>write!(formatter,""),
            Html=>write!(formatter,""),
            Css=>write!(formatter,""),
            Windows=>write!(formatter,""),
            Compressed=>write!(formatter,""),
            Archive=>write!(formatter,""),
            CompressedArchive=>write!(formatter,""),
            AndroidApp=>write!(formatter,""),
            Document=>write!(formatter,""),
            Spreadsheet=>write!(formatter,""),
            Presentation=>write!(formatter,""),
            Git=>write!(formatter,""),
            Vim=>write!(formatter,""),
            Font=>write!(formatter,""),
            Binary=>write!(formatter,""),
            Calc=>write!(formatter,""),
            Godot=>write!(formatter,"ﮧ"),
            Config=>write!(formatter,""),
            History=>write!(formatter,""),
            Regular=>write!(formatter,""),
        }
    }
}


pub struct FileCount {
    pre:String,
    content:String,
}
impl Display for FileCount {
    fn fmt(&self,formatter:&mut Formatter)->fmt::Result {
        if formatter.alternate() {
            write!(formatter,"{}{}",self.pre,self.content)
        } else {
            let tan=Rgb(0xD7,0xAF,0x87);
            let black=Rgb(0x0,0x0,0x0);
            let start_color=format!("{}{}",Bold,tan.bg_string());
            let mid_color=black.fg_string();
            let end_color=format!("{}{}",SReset,tan.fg_string());
            write!(formatter,"{}{}{}{}{}",start_color,self.pre,mid_color,self.content,end_color)
        }
    }
}
impl FileCount {
    fn new(len:usize,first:bool)->FileCount {
        let pre=if!first{""}else{""}.to_string();
        let content=if len==0 {
            format!("No files")
        } else if len==1 {
            format!("One file")
        } else {
            format!("{} files",len)
        };
        FileCount {
            pre,
            content,
        }
    }
}
struct Time {
    pre:String,
    content:String,
}
impl Display for Time {
    fn fmt(&self,formatter:&mut Formatter)->fmt::Result {
        if formatter.alternate() {
            write!(formatter,"{}{}",self.pre,self.content)
        } else {
            let blue=Rgb(0x00,0xaf,0xff);
            let black=Rgb(0x0,0x0,0x0);
            let start_color=format!("{}{}",Bold,blue.bg_string());
            let mid_color=black.fg_string();
            let end_color=format!("{}{}",SReset,blue.fg_string());
            write!(formatter,"{}{}{}{}{}",start_color,self.pre,mid_color,self.content,end_color)
        }
    }
}
impl Time {
    fn new(first:bool)->Time {
        let local=Local::now();
        let pre=if!first{""}else{""}.to_string();
        let content=format!("{}",local.format("%H:%M:%S"));
        Time {pre,content}
    }
}
struct Dir {
    pre:String,
    content:String,
}
impl Display for Dir {
    fn fmt(&self,formatter:&mut Formatter)->fmt::Result {
        if formatter.alternate() {
            write!(formatter,"{}{}",self.pre,self.content)
        } else {
            let green=Rgb(0x87,0xd7,0x87);
            let black=Rgb(0x0,0x0,0x0);
            let start_color=format!("{}{}",Bold,green.bg_string());
            let mid_color=black.fg_string();
            let end_color=format!("{}{}",SReset,green.fg_string());
            write!(formatter,"{}{}{}{}{}",start_color,self.pre,mid_color,self.content,end_color)
        }
    }
}
impl Dir {
    fn new(wd:String,first:bool)->Dir {
        let pre=if!first{""}else{""}.to_string();
        let content=format!("{}",wd);
        Dir {pre,content}
    }
}
struct Git {
    pre:String,
    content:String,
}
impl Display for Git {
    fn fmt(&self,formatter:&mut Formatter)->fmt::Result {
        if formatter.alternate() {
            write!(formatter,"{}{}",self.pre,self.content)
        } else {
            let grey=Rgb(0x22,0x22,0x22);
            let purple=Rgb(0xbf,0x00,0xFf);
            let start_color=format!("{}{}",Bold,grey.bg_string());
            let mid_color=purple.fg_string();
            let end_color=format!("{}{}",SReset,grey.fg_string());
            write!(formatter,"{}{}{}{}{}",start_color,self.pre,mid_color,self.content,end_color)
        }
    }
}
impl Git {
    fn new<T:AsRef<Path>>(pwd:T,first:bool)->Git {
        let pre=if!first{""}else{""}.to_string();
        let content=if let Ok(repo)=Repository::open(pwd) {
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
        Git {pre,content}
    }
}
struct DirList {
    items:Vec<ListFileType>,
}
impl Display for DirList {
    fn fmt(&self,formatter:&mut Formatter)->fmt::Result {
        if formatter.alternate() {
            for item in self.items.iter() {
                write!(formatter,"{:#}",item)?;
            }
            return Ok(());
        } else {
            for item in self.items.iter() {
                write!(formatter,"{}",item)?;
            }
            return Ok(());
        }
    }
}
impl DirList {
    fn len(&self)->usize {self.items.len()}
    fn new(pwd:String,uid:u32)->Result<DirList,io::Error> {
        let mut items=Vec::new();
        for item in read_dir(&pwd)? {
            let item=item?;
            let file_type=item.file_type()?;
            let name=item.file_name().into_string().expect("Found a non UTF8 filename");
            if file_type.is_symlink() {
                let path=item.path();
                if let Ok(link)=read_link(path) {
                    if let Ok(metadata)=link.metadata() {
                        items.push(ListFileType::new_symlink(name,metadata.file_type()));
                    }
                }
            } else if file_type.is_dir() {
                let item_count=read_dir(item.path())?.count();
                items.push(ListFileType::new_dir(name,pwd.clone(),item_count));
            } else if file_type.is_char_device() {
                items.push(ListFileType::new_char(name));
            } else if file_type.is_block_device() {
                items.push(ListFileType::new_block(name));
            } else {    // it is a file. we don't support sockets or FIFOs yet
                let metadata=item.metadata()?;
                let mode=metadata.mode();
                let any_exec=mode&0o001==0o001&&uid==metadata.uid();
                let user_exec=mode&0o100==0o100;
                let exec=any_exec|user_exec;
                items.push(ListFileType::new_file(name,exec));
            }
        }
        return Ok(DirList{items});
    }
    #[allow(dead_code)]
    fn get_items(&self,max_chars:usize,from:usize)->(String,usize) {
        let mut res=String::new();
        let mut chars_left=max_chars;
        let mut i=from;
        if i>=self.items.len() {return (String::new(),usize::MAX)}
        for item in self.items[i..].iter() {
            let uncolored=format!("{:#}",item);
            let len=uncolored.chars().count();
            if len>chars_left {
                break;
            }
            chars_left-=len;
            let colored=format!("{}",item);
            res.push_str(&colored);
            i+=1;
        }
        return (res,i);
    }
}
pub struct Status {
    pre:String,
    content:String,
}
impl Display for Status {
    fn fmt(&self,formatter:&mut Formatter)->fmt::Result {
        if self.content.len()>0 {
            if formatter.alternate() {
                write!(formatter,"{}{}",self.pre,self.content)
            } else {
                let red=Rgb(0xCC,0x00,0x00);
                let black=Rgb(0,0,0);
                let start_color=format!("{}{}",Bold,red.bg_string());
                let mid_color=black.fg_string();
                let end_color=format!("{}{}",SReset,red.fg_string());
                write!(formatter,"{}{}{}{}{}",start_color,self.pre,mid_color,self.content,end_color)
            }
        } else {
            return Ok(());
        }
    }
}
impl Status {
    fn new(status_raw:String,first:bool)->Status {
        let pre=if!first{""}else{""}.to_string();
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
        Status {pre,content}
    }
    fn is_active(&self)->bool {
        self.content.is_empty()
    }
}


const MAX_LINES:usize=3;


fn main() { // dirs command displays directory stack
    let args=args().collect::<Vec<String>>();
    let width=usize::from_str_radix(args[1].trim(),10).unwrap();
    //let height=usize::from_str_radix(args[2].trim(),10).unwrap();
    #[allow(unused)]
    {   // all the colors used
        let tan=Rgb(0xD7,0xAF,0x87);
        let green=Rgb(0x87,0xd7,0x87);
        let blue=Rgb(0x00,0xaf,0xff);
        let grey=Rgb(0x22,0x22,0x22);
        let pink=Rgb(0xFF,0x5F,0x87);
        let red=Rgb(0xCC,0x00,0x00);
        let purple=Rgb(0xbf,0x00,0xFf);
        let black=Rgb(0,0,0);
        let dark_blue=Rgb(0x34,0x65,0xA4);
        let cyan=Rgb(0x06,0x98,0x9A);
        let device_color=Rgb(0xC4,0xA0,0x00);
        let executable_color=Rgb(0x4E,0x9A,0x06);
    }
    // get env vars
    let env=vars().collect::<HashMap<String,String>>();
    // wd=(echo "$PWD"|sed "s@$HOME@~@g")
    let mut wd=env.get("PWD").unwrap().clone();
    // pwd=(echo "$PWD")
    let pwd=env.get("PWD").unwrap();
    let home=env.get("HOME").unwrap();
    let uid_str=String::from_utf8(Command::new("id").arg("-u").output().unwrap().stdout).unwrap();
    let uid_str=uid_str.trim();
    let uid=u32::from_str_radix(uid_str,10).unwrap();
    if wd.starts_with(home) {
        wd.replace_range(0..home.len(),"~");
    }
    // arg parsing
    if args.len()>=5 {
        if args[3]=="--prompt" {
            let status=args[4].trim().to_string();
            let status=Status::new(status,true);
            let dir_list=DirList::new(pwd.clone(),uid).unwrap();
            let file_count=FileCount::new(dir_list.len(),false);
            let time=Time::new(status.is_active());
            let dir=Dir::new(wd.clone(),false);
            let git=Git::new(pwd,false);
            let uncolored=format!("{:#}{:#}{:#}{:#}",time,dir,git,file_count);
            let len=uncolored.chars().count()+1;  // easiest way to get a count of the unicode characters
            print!("{}{}{}{}{}",status,time,dir,git,file_count);
            let white=Rgb(0xff,0xff,0xff);
            if width>len {
                let (line1,mut end)=dir_list.get_items(width-len,0);
                println!("{}",line1);
                for i in (0..MAX_LINES-2).rev() {
                    let x=dir_list.get_items(width-2,end);
                    let line=x.0;
                    end=x.1;
                    if line=="" {
                        println!("{}{}│",Reset.fg_str(),Reset.bg_str());
                    } else {
                        if i==0 {
                            println!("{}{}{}",line,Reset.bg_str(),white.fg_string());
                        } else {
                            println!("{}",line);
                        }
                    }
                }
            }
            return;
        }
        if args[3]=="--preexec" {
            print!("\r{}",ClearCurrentLine);
            for _ in 0..MAX_LINES {
                print!("{}{}",CursorUp(1),ClearCurrentLine);
            }
            // time to create another prompt
            let cyan=Rgb(0x06,0x98,0x9A);
            let black=Rgb(0,0,0);
            println!("{}{}{}{}{}{}{}",
                Time::new(true),
                cyan.bg_string(),   // bg
                black.fg_string(),  // fg
                args[4],
                cyan.fg_string(),
                Reset.bg_str(),
                Reset.fg_str(),
            );
            return;
        }
        if args[3]=="--postexec" {
            // unsupported so far
            return;
        }
    } else if args.len()>=4 {
    }
    // default single line prompt
    println!("{}{}└ﲒ ",Reset.fg_str(),Reset.bg_str());
}
