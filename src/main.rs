use aaron_asm::MachineState;
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
    let args: Vec<String> = std::env::args().collect();
    let mut arg_p: usize = 1;
    let mut compile_only = false;
    if args.len() > 1 {
        if args[arg_p] == "-c" {
            compile_only = true;
            arg_p += 1;
        }
        let file = File::open(&args[arg_p]);
        let mut file = file.if_error_then_exit();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let program = contents.parse();
        let program = program.if_error_then_exit();
        if compile_only {
            print!("{}", program);
        } else {
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            let mut machine = MachineState::new(&mut handle);
            println!("{}", machine.run(&program));
        }
    } else {
        eprintln!("Command line argument is invalid");
        std::process::exit(3);
    }
}
