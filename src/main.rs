use std::env;
use std::vec::Vec;
use std::fs;
use std::io::Read;


pub struct Code {
    
}

fn initialize() {

}

fn main() {
    let l:Vec<String> = env::args().collect();
    if l.len() == 1 {
        println!("usage: {} [source file]", l[0]);
        return;
    }

    let name = &l[1];


    let mut f = match fs::File::open(name) {
        Ok(f) => f,
        Err(E) => {
            println!("Failed to open file");
            return
        }
    };
    let mut code = String::new();

    f.read_to_string(&mut code);

    println!("{}", code);
}
