
extern crate aaron_asm;

use aaron_asm::*;
use std::fs::File;
use std::io::prelude::*;

trait ErrorExit<T, U> {
    fn if_error_then_exit(&self) -> &T;
}

impl<T, U> ErrorExit<T, U> for Result<T, U>
where
    U: std::fmt::Display,
{
    fn if_error_then_exit(&self) -> &T {
        match self {
            Ok(ref a) => a,
            Err(ref message) => {
                eprintln!("{}", message);
                std::process::exit(1);
            }
        }
    }
}

fn main() {
    let args : Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let file = File::open(&args[1]);
        let mut file = file.if_error_then_exit();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let program = contents.parse();
        let program = program.if_error_then_exit();
        let mut machine = MachineState::new();
        println!("{}", machine.run(&program));
    } else {
        eprintln!("Command line argument is invalid");
        std::process::exit(3);
    }
}
