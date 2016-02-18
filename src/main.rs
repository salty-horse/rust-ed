#![feature(str_char)]

use std::env;
use std::io;
use std::process;
use std::io::{BufRead, BufReader, Write};
use std::fs::File;
use std::collections::VecDeque;
use std::str::FromStr;

enum Mode {
    Command,
    Insert
}

#[derive(Debug)]
enum CommandType {
    Print,
    Quit
}

impl FromStr for CommandType {
    type Err = ();

    fn from_str(s: &str) -> Result<CommandType, ()> {
        match &*s {
            "p" => Ok(CommandType::Print),
            "q" => Ok(CommandType::Quit),
            //TODO replace with helpful messages like "?"
            x  => {
                println!("bad cmd: {:?}", x);
                Err(())
            }
        }
    }
}

struct Editor {
    mode: Mode,
    line_buffer: VecDeque<String>,
    //PROTIP this is 1-indexed!!!
    //that means always everywhere use it naturally
    //and always/only decrement for direct vec access
    //and $ is line_buffer.len(), NOT len-1
    //this meshes nicely with ed semantics as well
    //because something like 0i is meaningful
    //while 0p is nonsense
    current_line: usize
}

impl Editor {
    pub fn load(&mut self, path: &str) -> &mut Editor {
        let f = match File::open(path) {
            Ok(file) => file,
            Err(_) => panic!("bad file bro")
        };

        let mut file = BufReader::new(&f);
        for line in file.lines() {
            let l = line.unwrap();
            self.line_buffer.push_back(l);
        }

        self.current_line = self.line_buffer.len();

        self
    }

    pub fn handle_line(&mut self, line: &str) {
        match self.mode {
            Mode::Command => {
                let mut addr_mode = true;
                let mut addr_offset_mode = false;

                let mut left_addr = 0;
                let mut right_addr = 0;

                let mut p_flag = false;
                let mut n_flag = false;
                let mut l_flag = false;

                //FIXME address must handle arbitrary whitespace
                let mut chars = line.trim_left().chars().peekable();

                //address base
                match chars.peek() {
                    Some(&'.') => {
                        chars.next();
                        right_addr = self.current_line;
                    },
                    Some(&'$') => {
                        chars.next();
                        right_addr = self.line_buffer.len();
                    },
                    Some(&'%') | Some(&',') => {
                        chars.next();
                        left_addr = 1;
                        right_addr = self.line_buffer.len();
                    },
                    Some(n) if n.is_digit(10) => {
                        chars.next();
                        let mut num = (*n as isize);

                        loop {
                            match chars.peek() {
                                Some(n) if n.is_digit(10) => {
                                    chars.next();
                                    num = num * 10 + (*n as isize);
                                },
                                Some(_) => break,
                                None => panic!("this shouldn't happen")
                            }
                        }

                        //FIXME is all this isize/usize stuff sane
                        //annoying bc I was usize for array access
                        //but need signed int for < 0 during (but never after) math
                        //or... rearrange so that never matters? hm
                        if num < 1 || (num as usize) > self.line_buffer.len() {
                            panic!("return error");
                        }

                        right_addr = num as usize;
                    },
                    Some(&'\'') => {
                        chars.next();
                        match chars.next() {
                            Some(c) if c.is_alphabetic() => panic!("todo: marks"),
                            Some(_) | None => panic!("return error")
                        }
                    },
                    Some(&'/') | Some(&'?') => panic!("todo: regex mode"),
                    Some(_) => { ;},
                    None => panic!("this shouldn't happen")
                }
                
                println!("left: {}, right: {}", left_addr, right_addr);

                addr_offset_mode = true;
/* come back to this tmrw
                while addr_offset_mode {
                    match chars.peek() {
                        Some(
                    }
                }
*/
            },
            Mode::Insert => {
            }
        }
    }
}

impl Default for Editor {
    fn default() -> Editor {
        Editor {
            mode: Mode::Command,
            line_buffer: VecDeque::new(),
            current_line: 1,
        }
    }
}

fn main() {
    let mut ed = Editor { ..Default::default() };
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();
    let input = &mut String::new();

    match env::args().nth(1) {
        Some(arg) => ed.load(&arg),
        None => &mut ed //FIXME find out the right way to do this lol
    };

    loop {
        if ed.current_line < 1 {
            panic!("current line is {}--something has gone horribly wrong", ed.current_line);
        }

        input.clear();
        print!(":");
        stdout.lock().flush();

        stdin.read_line(input);
        ed.handle_line(&input);
    }
}
