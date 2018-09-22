use std::env;
use std::vec::Vec;
use std::fs;
use std::io::Read;
use std::collections::HashMap;
use std::str::Chars;


// TODO: ちゃんとパーサーを書く

// Instruction Pointer
pub struct Ip(u32);

pub struct Frame<'a> {
    retaddr: Ip,
    env: Env<'a>,
}

pub struct Env<'a> {
    stack: Vec<u8>,
    funcitons: HashMap<&'a str, Ip>,
}

pub struct Function<'a> {
    env: Vec<&'a mut Function<'a>>,
}

fn initialize() {}

fn run(code: &str) {}

fn collect_functions(code: &str) -> Vec<&mut Function> {
    let mut val = 0u32;
    let mut status = false;
    let mut ret: Vec<&mut Function> = Vec::new();

    for (i, c) in code.chars().enumerate() {
        match c {
            '{' =>  {

            },
            '}' => {

            },
            '0'...'9' => {

            },
            _ => {},
        }
    }
    ret
}

// code check
// checks parenthesis.
fn code_check(code: &str) -> bool {
    let mut val = 0;
    for (i, c) in code.chars().enumerate() {
        match c {
            '{' =>  {
                val += 1;
            },
            '}' => {
                val -= 1;
                if val < 0 {
                    println!("Syntax Error: Illegal close parenthesis at {}", i);
                    return false;
                }
            },
            _ => {},
        }
    }
    if val == 0 {
        true
    } else {
        println!("Syntax Error: unclosed parenthesis exists");
        false
    }
}

fn main() {
    let l: Vec<String> = env::args().collect();
    if l.len() == 1 {
        println!("usage: {} [source file]", l[0]);
        return;
    }

    let name = &l[1];

    let mut f = match fs::File::open(name) {
        Ok(f) => f,
        Err(E) => {
            println!("Failed to open file");
            return;
        }
    };
    let mut code = String::new();

    f.read_to_string(&mut code);

    initialize();
    if !code_check(&code) {
        return;
    }
    run(&code);
}
