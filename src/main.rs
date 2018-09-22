extern crate libc;
use std::env;
use std::vec::Vec;
use std::fs;
use std::io::Read;
use std::collections::HashMap;
use std::cell::RefCell;
use std::str::Chars;
use libc::getchar;


// TODO: ちゃんとパーサーを書く

// Instruction Pointer
#[derive(Copy, Clone, Debug)]
pub struct IP(u32);

impl IP {
    fn next(&mut self) {
        let &mut IP(v) = self;
        *self = IP(v + 1);
    }

    fn to_usize(&self) -> usize {
        let IP(v) = *self;
        v as usize
    }
}

fn to_v(c: char) -> i128 {
    ((c as u8) - ('0' as u8)) as i128
}

#[derive(Copy, Clone, Debug)]
pub struct FuncName(u32);

#[derive(Debug)]
pub struct Frame {
    pub ret_addr: IP,
    pub stack: Vec<i128>,
    pub env: Vec<ID>,
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


fn run(code: &[u8], functions: HashMap<ID, Function>, global: Vec<ID>) {
    let mut eip = IP(0);
    let mut stack: Vec<i128> = Vec::new();
    let mut env = Vec::new();
    let mut num_state = false;

    env.push(Frame{ret_addr: IP(0), stack: Vec::new(), env: Vec::new()});

    let mut current_scope = &global;

    loop {
        if eip.to_usize() >= code.len() {
            return;
        }
        assert!(env.len() > 0);
        let n = env.len();
        let inst = code[eip.to_usize()] as char;

        /*
        print!("eip {:?}. size {}: ", eip, stack.len());
        for c in stack.iter() {
            print!("{} ", c);
        }
        println!();
        */
        let old_eip = eip;
        let old_num_state = num_state;
        num_state = false;
        eip.next();
        match inst {
            '{' => {
                let frame: &mut Frame = &mut env[n - 1];
                stack.pop();
                let mut val = 1;
                for (i, c) in code[old_eip.to_usize()..].iter().enumerate() {
                    match *c as char {
                        '{' => {
                            val += 1;
                        },
                        '}' =>  {
                            val -= 1;
                            if val == 0 {
                                eip = IP((i + 1) as u32);
                                break;
                            }
                        },
                        _ => {},
                    }
                }

            },
            '}' => {
                eip = env[n - 1].ret_addr;
                env.pop();
            },
            '0'...'9' => {
                if old_num_state {
                    match stack.pop() {
                        Some(x) => {
                            stack.push(x * 10 + to_v(inst));
                            num_state = true;
                        },
                        None => {
                            stack.push(0);
                            num_state = true;
                        }
                    }
                } else {
                    stack.push(to_v(inst));
                    num_state = true;
                }
            },
            't' => {
                stack.pop();
            },
            ')' => {
                let frame: &mut Frame = &mut env[n - 1];
                match stack.pop() {
                    Some(x) => {
                        frame.stack.push(x);
                    },
                    None => {
                        panic!("Stack is empty!");
                    }
                }
            },
            '(' => {
                let frame: &mut Frame = &mut env[n - 1];
                match frame.stack.pop() {
                    Some(x) => {
                        stack.push(x);
                    },
                    None => {
                        panic!("Stack is empty!");
                    }
                }
            },
            'a' => {
                let frame: &mut Frame = &mut env[n - 1];
                while let Some(x) = frame.stack.pop() {
                    stack.push(x);
                }
            },
            'r' => {
                match (stack.pop(), stack.pop(), stack.pop()) {
                    (Some(a), Some(b), Some(c)) => {
                        stack.push(b);
                        stack.push(c);
                        stack.push(a);
                    },
                    _ => {
                        panic!("Stack size is smaller than 3 @ {}", old_eip.to_usize());
                    }
                }
            },
            's' => {
                match (stack.pop(), stack.pop()) {
                    (Some(a), Some(b)) => {
                        stack.push(b);
                        stack.push(a);
                    },
                    _ => {
                        panic!("Stack size is smaller than 2 @ {}", old_eip.to_usize());
                    }
                }
            },
            '+' | '-' | '*' | '/' | '>' | '<' | '=' => {
                match (stack.pop(), stack.pop()) {
                    (Some(a), Some(b)) => {
                        stack.push(
                            match inst {
                                '+' => a + b,
                                '-' => a - b,
                                '*' => a * b,
                                '/' => a / b,
                                '=' => (a == b) as i128,
                                '>' => (a > b) as i128,
                                '<' => (a < b) as i128,
                                _ => panic!("thinking face")
                            }
                        );
                    },
                    _ => {
                        panic!("Stack size is smaller than 2 @ {}", old_eip.to_usize());
                    }
                }
            },
            'i' => {
                match std::io::stdin().bytes().next() {
                    Some(Ok(c)) => {
                        stack.push(c as i128);
                    },
                    _ => {
                        panic!("stdin is closed @ {}", old_eip.to_usize());
                    }
                }
            },
            'o' => {
                match stack.pop() {
                    Some(c) => {
                        print!("{}", ((c % 256) as u8) as char);
                    },
                    None => {
                        panic!("Stack is Empty @ {}", old_eip.to_usize());;
                    }
                }
            },
            'n' => {
                match stack.pop() {
                    Some(c) => {
                        print!("{}", c);
                    },
                    None => {
                        panic!("Stack is Empty @ {}", old_eip.to_usize());;
                    }
                }
            },
            'c' => {
                match stack.pop() {
                    Some(c) => {
                        match functions.get(&ID(c as u32)) {
                            Some(entry) => {
                                let mut v = Vec::new();
                                let e = &entry.env;
                                v.extend(e.iter().cloned());
                                env.push(Frame{ret_addr: eip, stack: Vec::new(), env: v});
                                eip = (&entry).ip;
                                current_scope = e;
                            },
                            None => {
                                panic!("No such function: {} @ {}", c, old_eip.to_usize());
                            }
                        }
                    },
                    None => {
                        panic!("Stack is Empty @ {}", old_eip.to_usize());
                    }
                }
            },
            'b' => {
                match (stack.pop(), stack.pop(), stack.pop()) {
                    (Some(a), Some(b), Some(c)) => {
                        let x = if c == 0{ b } else { a };
                        match functions.get(&ID(x as u32)) {
                            Some(entry) => {
                                let mut v = Vec::new();
                                let e = &entry.env;
                                v.extend(e.iter().cloned());
                                env.push(Frame{ret_addr: eip, stack: Vec::new(), env: v});
                                eip = (&entry).ip;
                                current_scope = e;
                            },
                            None => {
                                panic!("No such function: {} @ {}", c, old_eip.to_usize());
                            }
                        }
                    },
                    _ => {
                        panic!("Stack size is smaller than 3 @ {}", old_eip.to_usize());
                    }
                }
            },
            _ =>  {
            }
        }
    }
}

fn collect_functions(code: &str) -> (HashMap<ID, Function>, Vec<ID>) {
    let mut val = 0u32;
    let mut env = Vec::new();
    let mut ret = HashMap::new();

    let mut id = 0u32;

    let mut nest = 0;

    env.push(Vec::new());
    for (i, c) in code.chars().enumerate() {
        match c {
            '{' => {
                let n = env.len();
                env[n - 1].push(ID(id));
                let mut v = Vec::new();
                for e in env.iter() {
                    v.extend(e.iter().cloned());
                }
                let mut fun = Function::new(v, IP(i as u32), FuncName(val), ID(id));
                ret.insert(ID(id), fun);
                env.push(Vec::new());
                id += 1;
                val = 0;
                nest += 1;
            },
            '}' => {
                nest -= 1;
                env.pop();
                val = 0;
            },
            '0'...'9' => {
                val = val * 10 + to_v(c) as u32;
            }
            _ => {
                val = 0;
            }
        }
    }
    (ret, env.pop().unwrap())
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
                name = name * 10 + to_v(c) as u32;
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
    let (functions, global) = collect_functions(&code);
    let bcode = code.as_bytes();
    run(bcode, functions, global);
}
