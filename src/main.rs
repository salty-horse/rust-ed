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
/*
                //find last char of address
                let mut idx = 0;
                for c in line.chars() {
                    if is_command(c) || c =='\n' {
                        break;
                    } else {
                        idx += 1;
                    }
                }

                //handle trivial but weird edge cases, otherwise parse
                let (left_addr, right_addr) = if idx == 0 {
                    (self.current_line, self.current_line)
                } else if 
*/

                let mut addr_mode = true;
                let mut addr_offset_mode = false;

                //various commands have their own default addresses
                //but 0 is valid input in some cases
                //so -1 communicates nil unambiguously
                let mut left_addr = -1;
                let mut right_addr = -1;
                let mut curr_addr = self.current_line;
                let mut expect_tail = false;

                let mut i = 0;
                while i < line.len() {
                    let c = line.char_at(i);

                    //ignore spacing, break on end of address
                    if c == ' ' || c == '\t' {
                        continue;
                    } else if is_command(c) || c == '\n' {
                        break;
                    }
                    
                match c {
                    '\n' | (_ if is_command(c)) => break,
                    //'\n' => break,
                    ' ' | '\t' => continue,
                    '.' => {
                        if expect_tail {
                            panic!("return error");
                        } else {
                            left_addr = right_addr;
                            right_addr = curr_addr;
                            expect_tail = true;
                        }
                    },
                    '$' => {
                        if expect_tail {
                            panic!("return error");
                        } else {
                            left_addr = right_addr;
                            right_addr = self.line_buffer.len();
                            expect_tail = true;
                        }
                    },
                    //like 1,$
                    '%' => {
                        if expect_tail {
                            panic!("return error");
                        } else {
                            left_addr = 1;
                            right_addr = self.line_buffer.len();
                            expect_tail = true;
                        }
                    },
                    ',' => {
                        //if true, delimiter. else like 1,$
                        if expect_tail {
                            expect_tail = false;
                        } else {
                            left_addr = 1;
                            right_addr = self.line_buffer.len();
                            expect_tail = true;
                        }
                    },
                    //FIXME this entire arm is cringe-inducing
                    n if n.is_digit(10) => {

                        let mut num = n.to_digit(10).unwrap() as isize;

                        //FIXME in C this would be a simple test on an assignment in parens
                        //look into while let, maybe that is what that is for
                        while line.char_at(i+1).is_some() && line.char_at(i+1).unwrap().is_digit(10) {
                            i += 1;
                            num = num * 10 + (line.char_at(i).unwrap().to_digit(10).unwrap() as isize);
                        }

                        //if true, positive offset. else assignment
                        //in both cases don't check validity until the end
                        //something like 1-500+500 is valid
                        if expect_tail {
                            right_addr += num;
                        } else {
                            left_addr = right_addr;
                            right_addr = num;
                            expect_tail = true;
                        }
/* TODO TODO when I am back, match +|- and do the same as the above basically
set bool for add/subtract, strip whitespace, load in the number if exists or plusminus 1 otherwise
also check whether it should change the current addr in place or if it just waits for the end
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

//FIXME actually switch on the specific chars that are commands?
fn is_command(c: char) -> bool {
    c.is_alphabetic() || c == '='
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
