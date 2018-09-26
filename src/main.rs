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

#[derive(Debug)]
pub enum Value {
    Int(i64),
    List(Vec<Value>),
}

#[derive(Debug)]
pub struct Setting {
    pub debug: bool,
}

impl Setting {
    fn new() -> Setting {
        Setting{debug: false}
    }
}

impl Value {
    fn print_inner(&self, val: bool) {
        match self {
            Value::Int(c) =>
            if val {
                print!("{}", c);
            } else {
                print!("{}", ((c % 256) as u8) as char);
            },
            Value::List(l) => {
                print!("[");
                for v in l.iter() {
                    v.print_inner(val);
                    print!(",");
                }
                print!("]");
            }
        }
    }
    pub fn print_val(&self) {
        self.print_inner(true);
    }
    pub fn print(&self) {
        self.print_inner(false);
    }
    pub fn clone(&self) -> Value {
        match &self {
            Value::Int(ref x) => Value::Int(*x),
            Value::List(ref l) => {
                let mut v = Vec::new();
                for x in l.iter() {
                    v.push(x.clone());
                }
                Value::List(v)
            }
        }
    }
}

fn to_v(c: char) -> i64 {
    ((c as u8) - ('0' as u8)) as i64
}

#[derive(Copy, Clone, Debug)]
pub struct FuncName(u32);

#[derive(Debug)]
pub struct Frame<'a> {
    pub ret_addr: IP,
    pub stack: Vec<Value>,
    pub env: &'a Vec<(ID, ID)>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ID(u32);

#[derive(Debug)]
pub struct Function {
    env: Vec<(ID, ID)>,
    ip: IP,
    name: FuncName,
    id: ID,
}

impl Function {
    pub fn new(env: Vec<(ID, ID)>, ip: IP, name: FuncName, id: ID) -> Function {
        let f = Function { env, ip, name, id };
        f
    }
    pub fn search_by_id(fs: &Vec<(ID, ID)>, id: &ID) -> Option<ID> {
        for (n, r) in fs.iter().rev() {
            if n == id {
                return Some(*r);
            }
        }
        None
    }
}


fn initialize() {}


fn run(code: &[u8], functions: & HashMap<ID, Function>, global: & Vec<(ID, ID)>, setting: &Setting, stack: &mut Vec<Value>, entry_point: IP) -> Option<Value> {
    let mut eip = entry_point;
    let mut env = Vec::new();
    let mut num_state = false;

    // u32 maxの長さがあるとだめです
    env.push(Frame{ret_addr: IP(u32::max_value()), stack: Vec::new(), env: &global});

    let mut current_scope = global;

    loop {
        if eip.to_usize() >= code.len() {
            return stack.pop();
        }

        assert!(env.len() > 0);
        let n = env.len();
        let inst = code[eip.to_usize()] as char;

        if setting.debug {
            print!("eip {:?}. size {}: ", eip, stack.len());
            for c in stack.iter() {
                c.print_val();
                print!(" ");
            }
            println!();
        }

        let old_eip = eip;
        let old_num_state = num_state;
        num_state = false;
        eip.next();
        match inst {
            '{' => {
                let frame: &mut Frame = &mut env[n - 1];
                stack.pop();
                let mut val = 1;
                let mut done = false;
                for (i, c) in code[old_eip.to_usize() + 1..].iter().enumerate() {
                    if done {
                        break;
                    }
                    match *c as char {
                        '{' => {
                            val += 1;
                        },
                        '}' =>  {
                            let cnt = i + old_eip.to_usize() + 2;
                            val -= 1;
                            if val == 0 {
                                eip = IP(cnt as u32);
                                done = true;
                            }
                        },
                        _ => {},
                    }
                }
            },
            '}' => {
                eip = env[n - 1].ret_addr;
                env.pop();
                if env.len() > 0 {
                    current_scope = &env[n - 2].env;
                } else {
                    return stack.pop();
                }
            },
            '0'...'9' => {
                if old_num_state {
                    match stack.pop() {
                        Some(Value::Int(x)) => {
                            stack.push(Value::Int(x * 10 + to_v(inst)));
                            num_state = true;
                        },
                        _ => {
                            panic!("something seems wrong.")
                        }
                    }
                } else {
                    stack.push(Value::Int(to_v(inst)));
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
                        stack.push(a);
                        stack.push(c);
                        stack.push(b);
                    },
                    _ => {
                        panic!("Stack size is smaller than 3 @ {}", old_eip.to_usize());
                    }
                }
            },
            's' => {
                match (stack.pop(), stack.pop()) {
                    (Some(a), Some(b)) => {
                        stack.push(a);
                        stack.push(b);
                    },
                    _ => {
                        panic!("Stack size is smaller than 2 @ {}", old_eip.to_usize());
                    }
                }
            },
            'd' => {
                match stack.pop() {
                    Some(v) => {
                        let w = (&v).clone();
                        stack.push(v);
                        stack.push(w);
                    },
                    None => {
                        panic!("Stack is Empty");
                    }
                }
            },
            '+' | '-' | '*' | '/' | '>' | '<' | '=' => {
                match (stack.pop(), stack.pop()) {
                    (Some(a), Some(b)) => {
                        match (a, b) {
                            (Value::Int(a), Value::Int(b)) => {
                                stack.push(Value::Int(
                                    match inst {
                                        '+' => a + b,
                                        '-' => a - b,
                                        '*' => a * b,
                                        '/' => a / b,
                                        '=' => (a == b) as i64,
                                        '>' => (a > b) as i64,
                                        '<' => (a < b) as i64,
                                        _ => panic!("thinking face")
                                    }
                                ));
                            },
                            (Value::List(mut a), Value::List(b)) => {
                                if inst == '+' {
                                    a.extend(b);
                                    stack.push(Value::List(a));
                                } else {
                                    panic!("{} is not supported to List @ {}", inst, old_eip.to_usize());
                                }
                            },
                            _ => {
                                panic!("type match error");
                            }
                        }
                    },
                    _ => {
                        panic!("Stack size is smaller than 2 @ {}", old_eip.to_usize());
                    }
                }
            },
            'i' => {
                match std::io::stdin().bytes().next() {
                    Some(Ok(c)) => {
                        stack.push(Value::Int(c as i64));
                    },
                    _ => {
                        panic!("stdin is closed @ {}", old_eip.to_usize());
                    }
                }
            },
            'o' => {
                match stack.pop() {
                    Some(v) => {
                        v.print();
                    },
                    None => {
                        panic!("Stack is Empty @ {}", old_eip.to_usize());;
                    }
                }
            },
            'n' => {
                match stack.pop() {
                    Some(v) => {
                        v.print_val();
                    },
                    None => {
                        panic!("Stack is Empty @ {}", old_eip.to_usize());;
                    }
                }
            },
            'c' => {
                match stack.pop() {
                    Some(Value::Int(c)) => {
                        let id = match Function::search_by_id(current_scope, &ID(c as u32)) {
                            Some(idx) => idx,
                            None => panic!("No such function {} in the current scope @ {}", c, old_eip.to_usize()),
                        };
                        match functions.get(&id) {
                            Some(entry) => {
                                let mut v: Vec<(ID, ID)> = Vec::new();
                                let e = &entry.env;
                                env.push(Frame{ret_addr: eip, stack: Vec::new(), env: current_scope});
                                eip = (&entry).ip;
                                eip.next(); // increment
                                current_scope = e;
                            },
                            None => {
                                panic!("No such function: {} @ {}", inst, old_eip.to_usize());
                            }
                        }
                    },
                    Some(Value::List(c)) => {
                        panic!("List is not supported @ {}", old_eip.to_usize());
                    },
                    None => {
                        panic!("Stack is Empty @ {}", old_eip.to_usize());
                    }
                }
            },
            'b' => {
                let a = stack.pop();
                let b = stack.pop();
                let c = stack.pop();
                match (a, b, c) {
                    (Some(Value::Int(a)), Some(Value::Int(b)), Some(Value::Int(c))) => {
                        let x = if c == 0 { b } else { a };
                        let id = match Function::search_by_id(current_scope, &ID(x as u32)) {
                            Some(idx) => idx,
                            None => panic!("No such function {} in the current scope @ {}", c, old_eip.to_usize()),
                        };
                        match functions.get(&id) {
                            Some(entry) => {
                                let e = &entry.env;
                                env.push(Frame{ret_addr: eip, stack: Vec::new(), env: current_scope});
                                eip = (&entry).ip;
                                eip.next();
                                current_scope = e;
                            },
                            None => {
                                panic!("No such function: {} @ {}", inst, old_eip.to_usize());
                            }
                        }
                    },
                    _ => {
                        panic!("Stack size is smaller than 3 @ {}", old_eip.to_usize());
                    }
                }
            },
            'l' => {
                stack.push(Value::List(Vec::new()));
            },
            'p' => {
                match (stack.pop(), stack.pop()) {
                    (Some(a), Some(Value::List(mut b))) => {
                        b.push(a);
                        stack.push(Value::List(b));
                    },
                    _ => {
                        panic!("Stack size is smaller than 2 @ {}", old_eip.to_usize());
                    }
                }
            },
            'e' => {
                println!();
            },
            '[' => {
                match stack.pop() {
                    Some(Value::List(mut l)) => {
                        if l.len() == 0 {
                            panic!("List len is 0");
                        }
                        let v = l[0].clone();
                        l.remove(0);
                        stack.push(Value::List(l));
                        stack.push(v);
                    },
                    _ => {
                        panic!("Stack top is not a list")
                    }
                }
            },
            ']' => {
                match stack.pop() {
                    Some(Value::List(mut l)) => {
                        match l.pop() {
                            Some(v) => {
                                stack.push(Value::List(l));
                                stack.push(v);
                            }

                            None => panic!("List len is 0"),
                        }
                    },
                    _ => {
                        panic!("Stack top is not a list")
                    }
                }
            },
            '_' => {
                match stack.pop() {
                    Some(Value::List(l)) => {
                        let v = l.len();
                        stack.push(Value::List(l));
                        stack.push(Value::Int(v as i64));
                    },
                    _ => {
                        panic!("Stack top is not a list")
                    }
                }
            },
            'f' => {
                match (stack.pop(), stack.pop()) {
                    (Some(Value::Int(x)), Some(Value::List(mut b))) => {
                        let id = match Function::search_by_id(current_scope, &ID(x as u32)) {
                            Some(idx) => idx,
                            None => panic!("No such function {} in the current scope @ {}", inst, old_eip.to_usize()),
                        };
                        match functions.get(&id) {
                            Some(entry) => {
                                let mut vec = Vec::new();
                                let mut eip = entry.ip;
                                eip.next();
                                for v in b.iter() {
                                    let mut stack = Vec::new();
                                    stack.push(v.clone());
                                    let ret = run(code, functions, global, setting, &mut stack, eip);
                                    match ret {
                                        Some(x) => vec.push(x),
                                        None => {},
                                    }
                                }
                                stack.push(Value::List(vec));
                            },
                            None => {
                                panic!("No such function: {} @ {}", inst, old_eip.to_usize());
                            }
                        }
                    },
                    _ => {
                        panic!("Stack size is smaller than 2 @ {}", old_eip.to_usize());
                    }
                }
            }
            _ =>  {
            }
        }
    }
}

fn collect_functions(code: &str) -> (HashMap<ID, Function>, Vec<(ID, ID)>) {
    let mut val = 0u32;
    let mut env = Vec::new();
    let mut ret = HashMap::new();

    let mut id = 0u32;
    let mut memo = Vec::new();

    let mut nest = 0;

    env.push(Vec::new());
    for (i, c) in code.chars().enumerate() {
        match c {
            '{' => {
                let n = env.len();
                env[n - 1].push((ID(val), ID(id)));
                let mut fun = Function::new(Vec::new(), IP(i as u32), FuncName(val), ID(id));
                ret.insert(ID(id), fun);
                memo.push(ID(id));
                env.push(Vec::new());
                id += 1;
                val = 0;
                nest += 1;
            },
            '}' => {
                let mut v = Vec::new();
                for e in env.iter() {
                    v.extend(e.iter().cloned());
                }
                let id = memo.pop().unwrap();

                ret.get_mut(&id).unwrap().env = v;
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

    let mut set = Setting::new();
    if l.len() >= 3 {
        for v in l.iter() {
            match v as &str {
                "d" => set.debug = true,
                _ => {},
            }
        }

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
    if set.debug {
        for f in functions.iter() {
            println!("{:?}", f);
        }
    }
    let bcode = code.as_bytes();
    let mut stack = Vec::new();
    run(bcode, &functions, &global, &set, &mut stack,IP(0));
}
