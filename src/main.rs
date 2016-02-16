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
                self.parse_command(line);
            },
            Mode::Insert => {
                print!("inputting! {}", line);
            }
        }
    }

    fn parse_command(&mut self, line: &str) {
        //FIXME this just strips off the \n, not really safe
        let (line, _) = line.split_at(line.len() - 1);
        let line = line.trim_left();

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

/*
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
