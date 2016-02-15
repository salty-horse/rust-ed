use std::env;
use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::fs::File;

struct Editor {
    line_buffer: VecDeque<String>,
    current_line: u64 //0-index
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
}

impl Default for Editor {
    fn default() -> Editor {
        Editor {
            line_buffer: VecDeque::new(),
            current_line: 1
        }
    }
}

fn main() {
    let mut ed = Editor { ..Default::default() };

    match env::args().nth(1) {
        Some(arg) => ed.load(&arg),
        None => &mut ed //FIXME find out the right way to do this lol
    };
}
