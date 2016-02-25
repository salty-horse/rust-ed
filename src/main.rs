#![feature(str_char)]

use std::io;
use std::io::{BufRead, BufReader, Write};
use std::fs::File;
use std::collections::{VecDeque, HashMap};
use std::collections::hash_map;
use std::str::FromStr;

enum Mode {
    Command,
    Insert
}

struct Editor {
    mode: Mode,
    line_buffer: VecDeque<String>,
    mark_hash: HashMap<char, usize>,
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
                let (addrs, idx) = match self.parse_addr(line) {
                    Ok(r) => r,
                    Err(e) => {
                        //TODO return to something so printing is done in one place
                        println!("?");
                        return;
                    }
                };

                //TODO Result<(),()> atm but make it useful later imo
                match self.parse_command(&line[idx..line.len()], addrs) {
                    Ok(_) => (),
                    Err(_) => {
                        println!("?");
                        return;
                    }
                }
            },
            Mode::Insert => {
            }
        }
    }

    //option tuple is addresses if any, usize is the *next index to read*, not the last read
    fn parse_addr(&mut self, line: &str) -> Result<(Option<(usize,usize)>,usize), ()> {
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
                 _ if is_command(c) => break,
                '\n'  => break,
                ' ' | '\t' => {
                    i +=1;
                    continue;
                },
                '.' => {
                    if expect_tail {
                        return Err(());
                    } else {
                        left_addr = right_addr;
                        right_addr = curr_addr;
                        expect_tail = true;
                        addrs += 1;
                    }
                },
                '$' => {
                    if expect_tail {
                        return Err(());
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
                        return Err(());
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
                    if expect_tail {
                        return Err(());
                    } else {
                        let m = line.char_at(i+1);

                        if m.is_alphabetic() && m.is_lowercase() {
                            i += 1;

                            match self.mark_hash.get(&m) {
                                //sanity check
                                Some(l) => {
                                    if *l > 0 && *l <= self.line_buffer.len() {
                                        left_addr = right_addr;
                                        right_addr = *l as isize;
                                        expect_tail = true;
                                        addrs += 1;
                                    } else {
                                        return Err(());
                                    }
                                },
                                None => return Err(())
                            }
                        } else {
                            return Err(());
                        }
                    }
                },
                '/' | '?' => panic!("todo: regex mode"),
                _ => { ;},
            }
        
            i += 1;
        } //end address parsing

        //validate
        if addrs > 0 {
            //negative is always an error, 0 is valid in some contexts
            if right_addr < 0 || (right_addr as usize) > self.line_buffer.len() {
                return Err(());
            }
        }
        if addrs > 1 {
            if left_addr < 0 || (left_addr as usize) > self.line_buffer.len() || left_addr > right_addr {
                return Err(());
            }
        }

        //return
        //FIXME this type shit I'm doing is dumb ugh lol
        //do this cleaner
        if addrs == 0 {
            Ok((None, i))
        } else if addrs == 1 {
            Ok((Some((right_addr as usize, right_addr as usize)), i))
        } else {
            Ok((Some((left_addr as usize, right_addr as usize)), i))
        }
    }

    fn parse_command(&mut self, line: &str, addrs: Option<(usize, usize)>) -> Result<(), ()> {
        //FIXME I was going to use an enum for commands but
        //it didn't seem to accomplish anything and just doubled the boilerplate
        //enumerating and/or modularizing funxtionality would be desirable
        //but maybe wait till we have code to divvy up before worrying
        //it _would_ be nice to like, link functions to enums or smth tho
        //
        //anyway until I actually split up functionality...
        //most commands should follow the same basic format
        // * match out addrs and set defaults if needed
        // * error out if 0 and 0 is invalid for that command
        // * apply whatever functionality to the line or lines
        // * set the new current line
        //if I did this right the addresses are already bounds-checked
        match line.char_at(0) {
            'd' => {
                let (left, right) = match addrs {
                    Some(t) => t,
                    None => (self.current_line, self.current_line)
                };

                if left <= 0 {
                    return Err(());
                }

                if left == right {
                    self.line_buffer.remove(right - 1);
                } else {
                    self.line_buffer.drain((left - 1)..right);
                }

                //set line to line after deleted range if exists
                //else to line before
                //we use left here because eg: 2,5d
                //what was line 6 (index 5) is now line 2 (index 1)
                //so we set current to 2. otoh: 2,$d
                //line 1 (index 0) is all that remains so left-1
                //FIXME FIXME and suddenly I realize none of my code can deal with an empty linebuffer
                //then again in ed proper seems anything not insertion fails w/ empty buffer
                self.current_line = if self.line_buffer.len() == 0 {
                    1
                } else if self.line_buffer.len() >= left {
                    left
                } else {
                    left - 1
                };

                let kv = self.mark_hash.clone().into_iter();
                for (k, v) in kv {
                    for i in left..(right + 1) {
                        if v == i {
                            self.mark_hash.remove(&k);
                        }
                    }
                }

                Ok(())
            },
            'i' => {
                let (_, right) = match addrs {
                    Some(t) => t,
                    None => (0, self.current_line)
                };

                self.mode = Command::Insert;

                self.current_line = right;

                Ok(())
            },
            //NOTE when I do join 1,1j is a noop but not an error
            'k' => {
                let (_, right) = match addrs {
                    Some(t) => t,
                    None => (0, self.current_line)
                };

                if right <= 0 {
                    return Err(());
                }

                self.mark_hash.insert(line.char_at(1), right);

                self.current_line = right;

                Ok(())
            },
            'p' => {
                let (left, right) = match addrs {
                    Some(t) => t,
                    None => (self.current_line, self.current_line)
                };

                if left <= 0 {
                    return Err(());
                }

                for i in (left - 1)..right {
                    println!("{}", self.line_buffer[i]);
                }

                self.current_line = right;

                Ok(())
            },
            '\n' => {
                //FIXME I hate this 0 is there anything like _ to use ugh
                let (_, right) = match addrs {
                    Some(t) => t,
                    None => (0, self.current_line)
                };

                println!("{}", self.line_buffer[right - 1]);

                self.current_line = right;

                Ok(())
            },
            _ => {
                println!("zzz sleep");

                Ok(())
            }
        }
    }
}

impl Default for Editor {
    fn default() -> Editor {
        Editor {
            mode: Mode::Command,
            mark_hash: HashMap::with_capacity(26),
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
