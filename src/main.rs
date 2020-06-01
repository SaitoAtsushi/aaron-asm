extern crate num_bigint;
extern crate num_traits;
extern crate peg;

extern crate aaron_asm;

use aaron_asm::*;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let args : Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let file = File::open(&args[1]);
        let mut file = match file {
            Ok(ref a) => a,
            Err(x) => { eprintln!("{}", x); std::process::exit(1); }
        };
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let program = compile(&contents);
        match program {
            Ok(program) => {
                let mut machine = MachineState::new();
                println!("{}", machine.run(&program));
            },
            Err(err) => {
                eprintln!("{:?}", err);
                std::process::exit(2);
            }
        }
    } else {
        eprintln!("Command line argument is invalid");
        std::process::exit(3);
    }
}
