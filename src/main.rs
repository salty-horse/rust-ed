#![feature(str_char)]

use std::{fmt, io};
use std::io::{BufRead, BufReader, Write};
use std::fs::File;
use std::collections::VecDeque;
use std::collections::hash_map;
use std::str::FromStr;

enum Mode {
    Command,
    Append
}

struct Marks {
    array: [Option<usize>; 26]
}

impl Marks {
    //init
    pub fn new() -> Marks {
        Marks { array: [None; 26] }
    }

    fn get_ix(c: char) -> usize {
        (c as usize) - ('a' as usize)
    }

    //set mark c for line l
    pub fn insert(&mut self, c: char, l: usize) {
        assert!('a' <= c && c <= 'z');
        self.array[Marks::get_ix(c)] = Some(l);
    }

    pub fn get(&mut self, c: char) -> Option<usize> {
        assert!('a' <= c && c <= 'z');
        self.array[Marks::get_ix(c)]
    }

    //increment all marks after inserted line
    pub fn add_line(&mut self, right: usize) {
        //this is a dumb hack but I think it is correct
        let right = if right == 0 {1} else {right};

        for m in self.array.iter_mut() {
            match *m {
                Some(v) => {
                    if v >= right {
                        *m = Some(v + 1);
                    }
                },
                None => {},
            }
        }
    }

    //delete marks on lines in range and decrease all after
    pub fn del_lines(&mut self, left: usize, right: usize) {
        //3,5d this is 3
        //so a mark on 6 shifts to 3
        let diff = right - left + 1;

        for m in self.array.iter_mut() {
            match *m {
                Some(v) => {
                    if v > right {
                        *m = Some(v - diff);
                    } else if v >= left && v <= right {
                        *m = None;
                    }
                },
                None => {},
            }
        }
    }
}

impl fmt::Debug for Marks {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map().entries(self.array.iter().enumerate().filter_map(
            |(i, m)| match *m {
                Some(v) => Some(((('a' as u8) + i as u8) as char, v)),
                None => None,
            }
        )).finish()
    }
}

struct Editor {
    mode: Mode,
    line_buffer: VecDeque<String>,
    marks: Marks,
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
    pub fn new() -> Editor {
        Editor {
            mode: Mode::Command,
            marks: Marks::new(),
            line_buffer: VecDeque::new(),
            current_line: 1,
        }
    }

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
            Mode::Append => {
                //TODO test commands at addr 0 properly
                //I'd written everything assuming this would set it back to 1 if 0
                //but turns out that leaving it is the correct behavior
                if line == ".\n" {
                    self.mode = Mode::Command;
                } else {
                    //TODO if this is slow for large buffers mb collect and do all at once
                    //note because current_line is 1-indexed this appends after line
                    //
                    //also NOTE
                    //I strip off newlines because that's what rust's load by lines thing does
                    //and it seems reasonable to just add the \n to every line again when saving
                    //but perhaps it makes sense to preseve them and just suppress on print
                    //(also fwiw I am strongly in favor of \n as line *terminator* not *seperator*
                    //
                    //also also NOTE
                    //windows \n\r garbage throws a wrench in this but idc rn
                    //get it working on normal oses first then worry about silly exceptions
                    //guess long-term just write a macro, closest we got to ifdefs here
                    self.line_buffer.insert(self.current_line, line[0..line.len()-1].to_owned());
                    self.current_line += 1;
                    self.marks.add_line(self.current_line);
                }
            }
        }
    }

    //option tuple is addresses if any, usize is the *next index to read*, not the last read
    fn parse_addr(&mut self, line: &str) -> Result<(Option<(usize,usize)>,usize), ()> {
        //I use isize because addresses can _temporarily_ go negative
        //eg -5000+5001 is perfectly valid
        //this shooouldn't cause problems... unless you have a file with > 2bn lines?
        let mut addrs = 0;
        let mut left_addr: isize = 0;
        let mut right_addr: isize = 0;
        let mut curr_addr: isize = self.current_line as isize;
        let mut expect_tail = false;

        //parse loop
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

                            match self.marks.get(m) {
                                //sanity check
                                Some(l) => {
                                    if l > 0 && l <= self.line_buffer.len() {
                                        left_addr = right_addr;
                                        right_addr = l as isize;
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
        //enumerating and/or modularizing functionality would be desirable
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
            'a' => {
                let (_, right) = match addrs {
                    Some(t) => t,
                    None => (0, self.current_line)
                };

                self.mode = Mode::Append;

                self.current_line = right;

                Ok(())
            },
            //FIXME ok uh
            //I should probably refactor a lil before this zzz
            //otherwise it is like literally "copy paste all of d and all of a"
            //'c' => { },
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

                self.marks.del_lines(left, right);

                Ok(())
            },
            'i' => {
                let (_, right) = match addrs {
                    Some(t) => t,
                    None => (0, self.current_line)
                };

                self.mode = Mode::Append;

                if right == 0 {
                    self.current_line = 0;
                } else {
                    self.current_line = right - 1;
                }

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

                let m = line.char_at(1);
                if m.is_alphabetic() && m.is_lowercase() {
                    self.marks.insert(m, right);
                } else {
                    return Err(());
                }

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
                //NOTE newline with no addr is equiv to +1p
                let (_, right) = match addrs {
                    Some(t) => t,
                    None => (0, self.current_line + 1)
                };

                if right <= 0 || right > self.line_buffer.len() {
                    return Err(());
                }

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

//FIXME actually switch on the specific chars that are commands?
fn is_command(c: char) -> bool {
    c.is_alphabetic() || c == '='
}

fn main() {
    let mut ed = Editor::new();
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
        if ed.current_line < 0 {
            panic!("current line is {}--something has gone horribly wrong", ed.current_line);
        }

        input.clear();
        print!(":");
        stdout.lock().flush();

        stdin.read_line(input);
        ed.handle_line(&input);
    }
}
