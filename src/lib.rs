pub use vm::MachineState;

mod compiler;
mod vm;

#[cfg(test)]
mod tests {
    extern crate num_bigint;
    extern crate num_traits;
    use super::vm;
    use num_bigint::BigInt;
    use std::str::FromStr;

    #[test]
    fn factorial_test() -> Result<(), Box<dyn std::error::Error>> {
        let program = include_str!("../testcase/factorial.asm").parse()?;
        let mut machine = vm::MachineState::new();
        assert_eq!(machine.run(&program), BigInt::from(120));
        Ok(())
    }

    #[test]
    fn square_test() -> Result<(), Box<dyn std::error::Error>> {
        let program = include_str!("../testcase/square.asm").parse()?;
        let mut machine = vm::MachineState::new();
        assert_eq!(machine.run(&program), BigInt::from(55));
        Ok(())
    }

    #[test]
    fn fibonacci_test() -> Result<(), Box<dyn std::error::Error>> {
        let program = include_str!("../testcase/fibonacci.asm").parse()?;
        let mut machine = vm::MachineState::new();
        assert_eq!(
            machine.run(&program),
            BigInt::from_str("354224848179261915075")?
        );
        Ok(())
    }
}
