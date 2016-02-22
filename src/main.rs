#![feature(str_char)]

use std::io;
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
    pub fn load(&mut self, path: &str) {
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
    }

    pub fn handle_line(&mut self, line: &str) {
        match self.mode {
            Mode::Command => {
                let addrs = self.parse_addr(line);
            },
            Mode::Insert => {
            }
        }
    }

    fn parse_addr(&mut self, line: &str) -> Result<(Option<(isize,isize)>,usize),&str> {
        //FIXME I use isize because addresses can _temporarily_ go negative
        //eg -5000+5001 is perfectly valid
        //this shooouldn't cause problems... unless you have a file with > 2bn lines?
        let mut addrs = 0;
        let mut left_addr: isize = 0;
        let mut right_addr: isize = 0;
        let mut curr_addr: isize = self.current_line as isize;
        let mut expect_tail = false;

        //address parse loop
        //TODO put this in a function
        let mut i = 0;
        while i < line.len() {
            let c = line.char_at(i);

            match c {
                '\n' | _ if is_command(c) => break,
                ' ' | '\t' => {
                    i +=1;
                    continue;
                },
                '.' => {
                    if expect_tail {
                        return Err("nope");
                    } else {
                        left_addr = right_addr;
                        right_addr = curr_addr;
                        expect_tail = true;
                        addrs += 1;
                    }
                },
                '$' => {
                    if expect_tail {
                        return Err("nope");
                    } else {
                        left_addr = right_addr;
                        right_addr = self.line_buffer.len() as isize;
                        expect_tail = true;
                        addrs += 1;
                    }
                },
                //like 1,$
                '%' => {
                    if expect_tail {
                        return Err("nope");
                    } else {
                        left_addr = 1;
                        right_addr = self.line_buffer.len() as isize;
                        expect_tail = true;
                        addrs += 2;
                    }
                },
                ',' => {
                    //if true, delimiter. else like 1,$
                    if expect_tail {
                        expect_tail = false;
                    } else {
                        left_addr = 1;
                        right_addr = self.line_buffer.len() as isize;
                        expect_tail = true;
                        addrs += 2;
                    }
                },
                ';' => {
                    //if true, delimiter. else like .,$
                    if expect_tail {
                        curr_addr = right_addr;
                        expect_tail = false;
                    } else {
                        left_addr = curr_addr;
                        right_addr = self.line_buffer.len() as isize;
                        expect_tail = true;
                        addrs += 2;
                    }
                },
                //FIXME this entire arm is cringe-inducing
                n if n.is_digit(10) => {

                    let mut num = n.to_digit(10).unwrap() as isize;

                    //FIXME in C this would be a simple test on an assignment in parens
                    //look into while let, maybe that is what that is for
                    while line.char_at(i+1).is_digit(10) {
                        i += 1;
                        num = num * 10 + (line.char_at(i).to_digit(10).unwrap() as isize);
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
                        addrs += 1;
                    }
                },
                s if s == '+' || s == '-' => {
                    let sign = if s == '+' {1} else {-1};

                    //after the +/- increment past whitespace
                    while line.char_at(i+1) == ' ' || line.char_at(i+1) == '\t' {
                        i += 1;
                    }

                    //now get number that may or may not follow the +/-
                    //bool is necessary because +0 is valid
                    //but if there's no num we act like +1
                    let mut num = 0;
                    let mut got_num = false;
                    while line.char_at(i+1).is_digit(10) {
                        i += 1;
                        num = num * 10 + (line.char_at(i).to_digit(10).unwrap() as isize);
                        got_num = true;
                    }

                    num = if got_num {num * sign} else {sign};

                    //similar to with number match
                    if expect_tail {
                        right_addr += num;
                    } else {
                        left_addr = right_addr;
                        right_addr = curr_addr + num;
                        expect_tail = true;
                        addrs += 1;
                    }
                },
                '\'' => {
                    if line.char_at(i+1).is_alphabetic() {
                        i += 1;
                        panic!("todo: marks");
                    } else {
                        return Err("nope");
                    }
                },
                '/' | '?' => panic!("todo: regex mode"),
                _ => { ;},
            }
        
            i += 1;
        } //end address parsing

        println!("left: {}, right: {}, addrs: {}", left_addr, right_addr, addrs);

        //validate
        if addrs > 0 {
            //negative is always an error, 0 is valid in some contexts
            if right_addr < 0 || (right_addr as usize) > self.line_buffer.len() {
                return Err("nope");
            }
        }
        if addrs > 1 {
            if left_addr < 0 || (left_addr as usize) > self.line_buffer.len() || left_addr > right_addr {
                return Err("nope");
            }
        }

        //return
        if addrs == 0 {
            Ok((None, i))
        } else if addrs == 1 {
            Ok((Some((right_addr, right_addr)), i))
        } else {
            Ok((Some((left_addr, right_addr)), i))
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

/*
    match env::args().nth(1) {
        Some(arg) => ed.load(&arg),
        None => &mut ed //FIXME find out the right way to do this lol
    };
*/
    ed.load("./testfile");
    println!("(loaded testfile)");

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
