use std::env;
use std::vec::Vec;
use std::fs;
use std::io::Read;
use std::collections::HashMap;
use std::cell::RefCell;
use std::str::Chars;


// TODO: ちゃんとパーサーを書く

// Instruction Pointer
#[derive(Copy, Clone, Debug)]
pub struct IP(u32);
#[derive(Copy, Clone, Debug)]
pub struct FuncName(u32);

#[derive(Debug)]
pub struct Frame<'a> {
    retaddr: IP,
    env: Env<'a>,
}

#[derive(Debug)]
pub struct Env<'a> {
    stack: Vec<u8>,
    functions: HashMap<&'a str, IP>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ID(u32);

#[derive(Debug)]
pub struct Function {
    env: Vec<ID>,
    ip: IP,
    name: FuncName,
    id: ID,
}

impl Function {
    pub fn new(env: Vec<ID>, ip: IP, name: FuncName, id: ID) -> Function {
        let f = Function { env, ip, name, id };
        f
    }
}

fn initialize() {}

fn run(code: &str) {}

fn collect_functions(code: &str) -> HashMap<ID, Function> {
    let mut val = 0u32;
    let mut env = Vec::new();
    let mut ret = HashMap::new();

    let mut id = 0u32;

    for (i, c) in code.chars().enumerate() {
        match c {
            '{' => {
                let mut v = Vec::new();
                v.extend(env.iter().cloned());
                let mut fun = Function::new(v, IP(i as u32), FuncName(val), ID(id));
                env.push(ID(id));
                ret.insert(ID(id), fun);
                id += 1;
                val = 0;
            }
            '}' => {
                env.pop();
                val = 0;
            }
            '0'...'9' => {
                val = val * 10 + ((c as u8) - ('0' as u8)) as u32;
            }
            _ => {
                val = 0;
            }
        }
    }
    ret
}

// code check
// checks parenthesis.
fn code_check(code: &str) -> bool {
    let mut val = 0;
    let mut name = 0u32;
    let mut status = false;

    for (i, c) in code.chars().enumerate() {
        match c {
            '{' => {
                if !status {
                    println!("Syntax Error: function must have its name. at {}", i)
                }
                status = false;
                name = 0;
                val += 1;
            }
            '}' => {
                val -= 1;
                name = 0;
                if val < 0 {
                    println!("Syntax Error: Illegal close parenthesis at {}", i);
                    return false;
                }
            }
            '0'...'9' => {
                status = true;
                name = name * 10 + ((c as u8) - ('0' as u8)) as u32;
            }
            _ => {
                status = false;
                name = 0;
            }
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
        Err(_) => {
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
