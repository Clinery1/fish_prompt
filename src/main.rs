use crossterm::{
    style::{
        Attribute,
        SetAttribute,
        SetForegroundColor,
        SetBackgroundColor,
        Color,
    },
    terminal::{
        Clear,
        ClearType,
        size as get_terminal_size,
    },
    cursor::MoveUp as CursorUp,
};
use std::{
    env::{
        vars,
        args,
    },
    collections::HashMap,
    process::Command,
};
use file_list_widget::*;
use pwd_widget::*;
use git_widget::*;
use time_widget::*;
use pipe_status_widget::*;


mod file_list_widget;
mod file_count_widget;
mod pwd_widget;
mod git_widget;
mod time_widget;
mod pipe_status_widget;


pub trait StatuslineWidget {
    fn display(&self,f:&mut StatuslineFormatter);
}
pub struct StatuslineFormatter {
    prev_bg:Color,
    first:bool,
    /// lines is a list of lengths of all the lines the prompt can cover from bottom (index 0) to
    /// top (index len-1)
    lines:Vec<usize>,
}
impl StatuslineFormatter {
    pub fn new(width:usize,lines:usize)->Self {
        Self {
            prev_bg:Color::Rgb{r:0,g:0,b:0},
            first:true,
            lines:vec![width;lines],
        }
    }
    pub fn line_chars_left(&self)->Option<usize> {
        self.lines.last().map(|l|*l)
    }
    pub fn write_section(&mut self,section:&str,fg:Color,bg:Color,attributes:Vec<Attribute>)->bool {
        let mut length=section.chars().count();
        // eprintln!("\"{}\":{}; {:?}",section,length,self.lines);
        if !self.first {
            length+=1;
        }
        let mut last=if let Some(l)=self.lines.last_mut() {
            l
        } else {return false};
        if length>*last {
            self.lines.pop();
            if let Some(last)=self.lines.last() {
                if *last<length {
                    return false;
                }
                if *last>0 {
                    self.write_section("",Color::Reset,Color::Reset,vec![]);
                }
                self.first=true;
                println!("{}{}",SetForegroundColor(Color::Reset),SetBackgroundColor(Color::Reset));
            } else {
                return false;
            }
            last=self.lines.last_mut().unwrap();
        }
        *last-=length;
        print!("{}{}",SetAttribute(Attribute::Reset),SetBackgroundColor(bg));
        if !self.first {
            print!("{}",SetForegroundColor(self.prev_bg));
        } else {
            self.first=false;
        }
        for attribute in attributes {
            print!("{}",SetAttribute(attribute));
        }
        print!("{}{}",SetForegroundColor(fg),section);
        self.prev_bg=bg;
        return true;
    }
}
impl Drop for StatuslineFormatter {
    fn drop(&mut self) {
        println!("{}{}",SetForegroundColor(Color::Reset),SetBackgroundColor(Color::Reset));
    }
}
#[derive(Copy,Clone)]
/// Copied from https://github.com/Clinery1/nebulous.nvim/blob/main/lua/nebulous/colors/night.lua
pub struct Colors {
    pub background:Color,
    pub red:Color,
    pub blue:Color,
    pub green:Color,
    pub purple:Color,
    pub yellow:Color,
    pub orange:Color,
    pub violet:Color,
    pub magenta:Color,
    pub pink:Color,
    pub white:Color,
    pub cyan:Color,
    pub aqua:Color,
    pub black:Color,
    pub grey:Color,
    pub light_grey:Color,
    pub dark_red:Color,
    pub dark_orange:Color,
    pub dark_blue:Color,
    pub dark_green:Color,
    pub dark_yellow:Color,
    pub dark_magenta:Color,
    pub dark_cyan:Color,
    pub dark_aqua:Color,
    pub dark_grey:Color,
    pub dark_grey_2:Color,
    pub custom_1:Color,
    pub custom_2:Color,
    pub custom_3:Color,
}
impl Default for Colors {
    fn default()->Self {
        Self {
            background:     Color::Rgb{r:0x00,g:0x00,b:0x00},
            red:            Color::Rgb{r:0xFB,g:0x46,b:0x7B},
            blue:           Color::Rgb{r:0x0B,g:0xA8,b:0xE2},
            green:          Color::Rgb{r:0xB8,g:0xEE,b:0x92},
            purple:         Color::Rgb{r:0x97,g:0x5E,b:0xEC},
            yellow:         Color::Rgb{r:0xFF,g:0xCC,b:0x00},
            orange:         Color::Rgb{r:0xff,g:0x8d,b:0x03},
            violet:         Color::Rgb{r:0xF2,g:0x81,b:0xF2},
            magenta:        Color::Rgb{r:0xF9,g:0x5C,b:0xE6},
            pink:           Color::Rgb{r:0xDB,g:0x73,b:0xDA},
            white:          Color::Rgb{r:0xCE,g:0xD5,b:0xE5},
            cyan:           Color::Rgb{r:0x80,g:0xa0,b:0xff},
            aqua:           Color::Rgb{r:0x00,g:0xD5,b:0xA7},
            black:          Color::Rgb{r:0x0b,g:0x10,b:0x15},
            grey:           Color::Rgb{r:0x49,g:0x46,b:0x46},
            light_grey:     Color::Rgb{r:0x13,g:0x19,b:0x1F},
            dark_red:       Color::Rgb{r:0xFD,g:0x2E,b:0x6A},
            dark_orange:    Color::Rgb{r:0xDE,g:0x7A,b:0x00},
            dark_blue:      Color::Rgb{r:0x00,g:0x7E,b:0xD3},
            dark_green:     Color::Rgb{r:0x5E,g:0xB9,b:0x5D},
            dark_yellow:    Color::Rgb{r:0xB9,g:0x95,b:0x02},
            dark_magenta:   Color::Rgb{r:0xFE,g:0x92,b:0xE1},
            dark_cyan:      Color::Rgb{r:0x56,g:0xD6,b:0xD6},
            dark_aqua:      Color::Rgb{r:0x00,g:0xA5,b:0x82},
            dark_grey:      Color::Rgb{r:0x55,g:0x55,b:0x55},
            dark_grey_2:    Color::Rgb{r:0x82,g:0x89,b:0x89},
            custom_1:       Color::Rgb{r:0x2D,g:0x30,b:0x36},
            custom_2:       Color::Rgb{r:0xAF,g:0xFD,b:0xF1},
            custom_3:       Color::Rgb{r:0xE2,g:0xE7,b:0xE6},
        }
    }
}


const MAX_LINES:usize=3;


fn main() { // dirs command displays directory stack
    let args=args().collect::<Vec<String>>();
    let (width,height)=get_terminal_size().unwrap();
    let (width,_height)=(width as usize,height as usize);
    #[allow(unused)]
    {   // all the colors used
        let tan=Color::Rgb{r:0xD7,g:0xAF,b:0x87};
        let green=Color::Rgb{r:0x87,g:0xd7,b:0x87};
        let blue=Color::Rgb{r:0x00,g:0xaf,b:0xff};
        let grey=Color::Rgb{r:0x22,g:0x22,b:0x22};
        let pink=Color::Rgb{r:0xFF,g:0x5F,b:0x87};
        let red=Color::Rgb{r:0xCC,g:0x00,b:0x00};
        let purple=Color::Rgb{r:0xbf,g:0x00,b:0xFf};
        let black=Color::Rgb{r:0,g:0,b:0};
        let dark_blue=Color::Rgb{r:0x34,g:0x65,b:0xA4};
        let cyan=Color::Rgb{r:0x06,g:0x98,b:0x9A};
        let device_color=Color::Rgb{r:0xC4,g:0xA0,b:0x00};
        let executable_color=Color::Rgb{r:0x4E,g:0x9A,b:0x06};
    }
    // get env vars
    let env=vars().collect::<HashMap<String,String>>();
    // wd=(echo "$PWD"|sed "s@$HOME@~@g")
    // pwd=(echo "$PWD")
    let pwd=env.get("PWD").unwrap();
    let home=env.get("HOME").unwrap();
    let uid_str=String::from_utf8(Command::new("id").arg("-u").output().unwrap().stdout).unwrap();
    let uid_str=uid_str.trim();
    let uid=u32::from_str_radix(uid_str,10).unwrap();
    let wd=env.get("PWD").unwrap().replacen(home,"~",1);
    let mut execute_oneline=true;
    // arg parsing
    if args.len()>=3 {
        if args[1]=="--prompt" {
            let mut formatter=StatuslineFormatter::new(width,MAX_LINES-1);
            execute_oneline=false;
            let status=args[2].trim().to_string();
            let status=PipeStatus::new(&status);
            let time=Time::new();
            let dir=Pwd::new(&wd);
            let git=Git::new(pwd);
            let dirs=DirList::new(&pwd,uid).unwrap();
            let file_count=dirs.file_count();
            status.display(&mut formatter);
            time.display(&mut formatter);
            dir.display(&mut formatter);
            git.display(&mut formatter);
            file_count.display(&mut formatter);
            dirs.display(&mut formatter);
            formatter.write_section("",Color::Reset,Color::Reset,vec![]);
            for line in formatter.lines.iter() {
                if *line==width {
                    print!("\n│");
                }
            }
        } else if args[1]=="--preexec" {
            let mut formatter=StatuslineFormatter::new(width,MAX_LINES-1);
            execute_oneline=false;
            print!("\r{}",Clear(ClearType::CurrentLine));
            for _ in 0..MAX_LINES {
                print!("{}{}",CursorUp(1),Clear(ClearType::CurrentLine));
            }
            // time to create another prompt
            let time=Time::new();
            let dir=Pwd::new(&wd);
            let git=Git::new(pwd);
            let colors=Colors::default();
            time.display(&mut formatter);
            dir.display(&mut formatter);
            git.display(&mut formatter);
            formatter.write_section(&args[2],colors.black,colors.dark_aqua,vec![]);
            formatter.write_section("",Color::Reset,Color::Reset,vec![]);
        } else if args[1]=="--postexec" {
            execute_oneline=false;
            // unsupported so far
        }
    }
    if execute_oneline {
        // default single line prompt
        println!("{}{}└ﲒ ",SetForegroundColor(Color::Reset),SetBackgroundColor(Color::Reset));
    }
}
