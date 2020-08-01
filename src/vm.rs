use super::compiler::*;
extern crate num_bigint;
extern crate num_traits;
use num_traits::ToPrimitive;
const MEMORY_LIMIT: usize = 100000;

pub struct MachineState<'a, T: std::io::Write> {
    registers: Vec<Number>,
    program_counter: Number,
    output: &'a mut T,
}

trait OperandEval<T> {
    fn eval<'a>(&'a mut self, i: &'a T) -> Number;
}

impl<'b, T: std::io::Write> OperandEval<Index> for MachineState<'b, T> {
    fn eval<'a>(&'a mut self, i: &'a Index) -> Number {
        match &i {
            &Index::Direct(ref x) => x.clone(),
            &Index::Indirect(ref x) => self.register(x),
        }
    }
}

impl<'b, T: std::io::Write> OperandEval<Value> for MachineState<'b, T> {
    fn eval<'a>(&'a mut self, i: &'a Value) -> Number {
        match &i {
            &Value::Immediate(ref x) => x.clone(),
            &Value::Register(ref x) => self.register(x),
            &Value::Pointer(ref x) => self.register(&self.register(x)),
            &Value::ProgramCounter => self.program_counter.clone(),
            _ => panic!("Invalid operand"),
        }
    }
}

impl<'b, T: std::io::Write> OperandEval<Address> for MachineState<'b, T> {
    fn eval<'a>(&'a mut self, i: &'a Address) -> Number {
        match &i {
            &Address::Immediate(ref x) => x.clone(),
            &Address::Register(ref x) => self.register(x),
            &Address::ProgramCounter => self.program_counter.clone(),
            _ => panic!("Invalid operand"),
        }
    }
}

impl<'b, T: std::io::Write> MachineState<'b, T> {
    pub fn new(o: &'b mut T) -> MachineState<'b, T> {
        MachineState {
            registers: vec![Number::from(0)], // Vec::with_capacity(FIRST_MEMORY_SIZE),
            program_counter: Default::default(),
            output: o,
        }
    }

    pub fn run(&mut self, program: &Program) -> Number {
        loop {
            let program_counter = self.program_counter.to_usize();
            let program_counter = match program_counter {
                None => {
                    eprintln!("Invalid program counter {}", self.program_counter);
                    std::process::exit(4);
                }
                Some(ref a) if a > &program.len() => {
                    eprintln!("Invalid program counter {}", self.program_counter);
                    std::process::exit(4);
                }
                Some(a) => a,
            };
            match &program[program_counter] {
                &Statement::Incr(ref index, ref value) => {
                    self.program_counter += 1;
                    let index = &self.eval(index);
                    if index.sign() != num_bigint::Sign::Minus {
                        let value = &self.eval(value);
                        *self.register_mut(index) += value;
                    }
                }
                &Statement::Decr(ref index, ref address, ref value) => {
                    self.program_counter += 1;
                    let index = &self.eval(index);
                    let address = self.eval(address);
                    let value = &self.eval(value);
                    if self.register(index) >= *value {
                        *self.register_mut(index) -= value;
                    } else {
                        self.program_counter = address;
                    }
                }
                &Statement::Save(ref index, ref value) => {
                    self.program_counter += 1;
                    let index = &self.eval(index);
                    let value = self.eval(value);
                    *self.register_mut(index) = value;
                }
                &Statement::Putc(ref value) => {
                    self.program_counter += 1;
                    let value = self.eval(value);
                    write!(self.output, "{}", std::char::from_u32(value.to_u32().unwrap()).unwrap()).unwrap();
                }
                &Statement::Putn(ref value) => {
                    self.program_counter += 1;
                    let value = self.eval(value);
                    write!(self.output, "{}", value).unwrap();
                }
                &Statement::Halt => {
                    break;
                }
            }
        }

        self.register(&Number::from(0))
    }

    fn register(&self, num: &Number) -> Number {
        let num = num.to_usize();
        match num {
            Some(x) => {
                if self.registers.len() <= x {
                    Number::from(0)
                } else {
                    self.registers[x].clone()
                }
            }
            None => Number::from(0),
        }
    }

    fn register_mut(&mut self, num: &Number) -> &mut Number {
        let num = num.to_usize();
        match num {
            Some(x) => {
                if x > MEMORY_LIMIT {
                    eprintln!("Too big register number");
                    std::process::exit(5);
                }
                if self.registers.len() <= x {
                    self.registers.resize_with(x + 1, Default::default);
                }
                &mut self.registers[x]
            }
            None => {
                eprintln!("Too big register number");
                std::process::exit(5);
            }
        }
    }
}
