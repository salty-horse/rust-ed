#![feature(plugin)]
#![plugin(regex_macros)]

#![feature(str_char)]

extern crate regex;

use std::env;
use std::io;
use std::process;
use std::io::{BufRead, BufReader, Write};
use std::fs::File;
use std::collections::VecDeque;
use std::str::FromStr;
use regex::Regex;

enum Mode {
    Command,
    Insert
}

#[derive(Debug)]
enum CommandType {
    Print,
    Quit
}

enum ParseMode {
    StartAddress,
    EndAddress,
    Command,
    Rest
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
    current_line: usize, //0-index
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

        self.current_line = self.line_buffer.len() - 1;

        self
    }

    pub fn handle_line(&mut self, line: &str) {
        match self.mode {
            Mode::Command => {
                //self.parse_command(line);
                let line = line.trim_left();

                //FIXME all this probably doesn't work on windows
                //worry about it later
                if line == "\n" {
                    if self.current_line < self.line_buffer.len() {
                        self.current_line += 1;
                        println!("{}", self.line_buffer[self.current_line]);

                        return;
                    } else {
                        //XXX error
                        println!("?");

                        return;
                    }
                } else if line.len() == 2 && line.char_at(0).is_alphabetic() {
                    //FIXME damn girl string slices or chars pick one or the other
                    let cmd = match CommandType::from_str(line.split_at(1).0) {
                        Ok(cmd) => cmd,
                        Err(_) => {
                            //XXX error
                            println!("?");
                            return;
                        }
                    };

                    println!("doing cmd {:?}", cmd);
                    return;
                }

                let mut idx = 0;

                while idx < line.len() {
                    if line.char_at(idx).is_alphabetic()
                    && (idx == 0 || line.char_at(idx - 1) != '\'') {
                        break; //got command
                    }
                    idx += 1; 
                }

                let (addr, rest) = line.split_at(idx);
                let (cmd, rest) = rest.split_at(1);

                println!("addr {:?} cmd {:?} rest {:?}", addr, cmd, rest);
            },
            Mode::Insert => {
                print!("inputting! {}", line);
            }
        }
    }

    fn parse_command(&mut self, line: &str) {
        let re = regex!(concat!(
            r"^\s*",            //leading whitespace
            r"([0-9]+|\.)?",    //first address
            r"(,|;)?",          //address seperator
            r"([0-9]+|\.)?",    //second address
            r"([a-zA-Z])?",     //command
            r"(.+)?",           //rest
            r"[\n\r]*$"         //line terminator
        ));
        let caps = match re.captures(line) {
            Some(caps) => caps,
            None => return //FIXME parse error
        };

        let start_addr = caps.at(1);
        let end_addr = caps.at(2);
        let cmd = caps.at(3);
        let rest = caps.at(4);

        for cap in caps.iter() {
            println!("cap: {:?}", cap);
        }
/*
        let mut parse_mode = ParseMode::StartAddress;
        //FIXME I should be using str slices, fix it _after_ it works tho
        let mut start_address = String::new();
        let mut end_address = String::new();
        let mut command = String::new();
        let mut rest = String::new();

        for c in line.chars() {
            if c == '\n' {
                return;
            }

            if parse_mode == ParseMode::StartAddress {
                match c {
                    ',' => {
                        parse_mode = ParseMode::EndAddress;
                        continue;
                    },
            }
        }

        //let mut start_address = -1;
        //let mut end_address = -1;
        let mut idx = 0;
        println!("{}", line.len());
        while idx < line.len() && !line.char_at(idx).is_alphabetic() {
            println!("checking: {}", line.char_at(idx));

            idx += 1;
        }
        println!("{}", line.len());

        let (addr, cmd, rest) = if idx < line.len() {
            let (addr, rest) = line.split_at(idx);
            let (cmd, rest) = rest.split_at(1);

            (addr, cmd, rest)
        } else {
            return; //TODO parse error
        };
        
        println!("{:?} - {:?} - {:?}" , addr, cmd, rest);

        let cmd_type = CommandType::from_str(line).unwrap();

        match cmd_type {
            CommandType::Print => println!("printing~"),
            CommandType::Quit => process::exit(0)
        }
*/
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
        input.clear();
        print!(":");
        stdout.lock().flush();

        stdin.read_line(input);
        ed.handle_line(&input);
    }
}
