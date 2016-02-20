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
                //FIXME this is dumb but I am tired
                let c = if chars.peek().is_some() {
                    *(chars.peek().unwrap())
                } else {
                    panic!("this shouldn't happen")
                };
                    
                match c {
                    '.' => {
                        chars.next();
                        right_addr = self.current_line;
                    },
                    '$' => {
                        chars.next();
                        right_addr = self.line_buffer.len();
                    },
                    '%' | ',' => {
                        chars.next();
                        left_addr = 1;
                        right_addr = self.line_buffer.len();
                    },
                    //this entire arm is cringe-inducing
                    n if n.is_digit(10) => {
                        chars.next();
                        let mut num = n.to_digit(10).unwrap() as isize;

                        loop {
                            println!("n: {}, num: {}", n, num);
                            //FIXME this is dumb
                            let n = if chars.peek().is_some() {
                                *(chars.peek().unwrap())
                            } else {
                                panic!("this shouldn't happen")
                            };

                            match n {
                                n if n.is_digit(10) => {
                                    chars.next();
                                    num = num * 10 + (n.to_digit(10).unwrap() as isize);
                                },
                                _ => break,
                                //None => panic!("this shouldn't happen")
                            }
                        }

                        //FIXME is all this isize/usize stuff sane
                        //annoying bc I was usize for array access
                        //but need signed int for < 0 during (but never after) math
                        //or... rearrange so that never matters? hm
                        if num < 1 || (num as usize) > self.line_buffer.len() {
                            println!("num is {}", num);
                            panic!("return error");
                        }

                        right_addr = num as usize;
                    },
                    '\'' => {
                        chars.next();
                        let c = chars.next();
                        if c.is_some() {
                            let c = c.unwrap();
                            if c.is_alphabetic() {
                                panic!("todo: marks");
                            } else {
                                panic!("return error");
                            }
                        }
                    },
                    '/' | '?' => panic!("todo: regex mode"),
                    _ => { ;},
                    //None => panic!("this shouldn't happen")
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
