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
    io,
};
use crate::{
    file_count_widget::*,
    StatuslineWidget,
    StatuslineFormatter,
    Colors,
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
    fn new_dir(name:String,pwd:&str,items:usize)->ListFileType {
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
    fn new_symlink(name:String,file_type:Option<FileType>,broken:bool)->ListFileType {
        let icon;
        if let Some(file_type)=file_type {
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
        } else {
            if broken {
                icon=SymlinkIcon::Broken;
            } else {
                icon=SymlinkIcon::Unknown;
            }
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
impl StatuslineWidget for ListFileType {
    fn display(&self,f:&mut StatuslineFormatter) {
        let colors=Colors::default();
        match self {
            Self::File {name,primary,secondary,executable}=>{
                let mut s=format!(" {}",primary);
                if let Some(sec)=secondary {
                    s.push_str(&sec.to_string());
                }
                s.push_str(name);
                let fg=if *executable {
                    colors.green
                } else {
                    colors.white
                };
                let bg=colors.background;
                f.write_section(&s,fg,bg,vec![]);
            },
            Self::Directory{name,icon,items}=>{
                let s=if *items>0 {
                    format!(" {}{}[{}]",icon,name,items)
                } else {
                    format!(" {}{}",icon,name)
                };
                let fg=colors.white;
                let bg=colors.dark_blue;
                f.write_section(&s,fg,bg,vec![]);
            },
            Self::Symlink{name,icon}=>{
                let s=format!(" {}{}",icon,name);
                f.write_section(&s,colors.black,colors.cyan,vec![]);
            },
            Self::Character{name}=>{
                let s=format!(" {}",name);
                f.write_section(&s,colors.dark_yellow,colors.background,vec![]);
            },
            Self::Block{name}=>{
                let s=format!(" {}",name);
                f.write_section(&s,colors.dark_yellow,colors.background,vec![]);
            },
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
    Broken,
    Unknown,
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
            Broken=>write!(formatter,""),
            Unknown=>write!(formatter,""),
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
            "vim"|"vimrc"|"nvimrc"|"nvim"=>Vim,
            "js"|"ts"=>Javascript,
            "py"=>Python,
            "html"=>Html,
            "css"=>Css,
            "exe"|"dll"=>Windows,
            "gz"|"zstd"|"xz"=>Compressed,
            "tar"=>Archive,
            "zip"|"rar"=>CompressedArchive,
            n if n.ends_with("history")=>History,
            _=>Regular,
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


pub struct DirList {
    items:Vec<ListFileType>,
}
impl StatuslineWidget for DirList {
    fn display(&self,f:&mut StatuslineFormatter) {
        for i in self.items.iter() {
            i.display(f);
        }
    }
}
impl DirList {
    fn len(&self)->usize {self.items.len()}
    pub fn file_count(&self)->FileCount {FileCount::new(self.len())}
    pub fn new(pwd:&str,uid:u32)->Result<DirList,io::Error> {
        let mut items=Vec::new();
        if let Ok(dir_iter)=read_dir(&pwd) {
            for item in dir_iter {
                if let Ok(item)=item {
                    let file_type=item.file_type()?;
                    let name=item.file_name().into_string().expect("Found a non UTF8 filename");
                    if file_type.is_symlink() {
                        let path=item.path();
                        if let Ok(link)=read_link(path) {
                            if let Ok(metadata)=link.metadata() {
                                items.push(ListFileType::new_symlink(name,Some(metadata.file_type()),false));
                            } else {
                                items.push(ListFileType::new_symlink(name,None,false));
                            }
                        } else {
                            items.push(ListFileType::new_symlink(name,None,true));
                        }
                    } else if file_type.is_dir() {
                        let item_count=if let Ok(dir)=read_dir(item.path()) {
                            dir.count()
                        } else {
                            0
                        };
                        items.push(ListFileType::new_dir(name,pwd,item_count));
                    } else if file_type.is_char_device() {
                        items.push(ListFileType::new_char(name));
                    } else if file_type.is_block_device() {
                        items.push(ListFileType::new_block(name));
                    } else {    // assume the item is a regular file. we don't support sockets or FIFOs yet
                        let metadata=item.metadata()?;
                        let mode=metadata.mode();
                        let any_exec=mode&0o001==0o001&&uid==metadata.uid();
                        let user_exec=mode&0o100==0o100;
                        let exec=any_exec|user_exec;
                        items.push(ListFileType::new_file(name,exec));
                    }
                }
            }
        }
        return Ok(DirList{items});
    }
}
