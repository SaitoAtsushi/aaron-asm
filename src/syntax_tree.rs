extern crate num_bigint;
extern crate num_traits;
pub type Number = num_bigint::BigInt;

use std::fmt;
use std::option::Option;

#[derive(Clone)]
pub enum Index {
    Direct(Number),
    Indirect(Number),
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Index::Direct(ref n) => write!(f, "{}", n),
            Index::Indirect(ref n) => write!(f, "[{}]", n),
        }
    }
}

#[derive(Clone)]
pub enum Value {
    Immediate(Number),
    Register(Number),
    Pointer(Number),
    Label(String),
    ProgramCounter,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Immediate(ref n) => write!(f, "{}", n),
            Value::Register(ref n) => write!(f, "[{}]", n),
            Value::Pointer(ref n) => write!(f, "[[{}]]", n),
            Value::Label(ref n) => write!(f, "{}", n),
            Value::ProgramCounter => write!(f, "pc"),
        }
    }
}

impl Value {
    fn solve(&self, labels: &HashMap<&String, Number>, pc: usize) -> Option<Value> {
        match self {
            Value::Label(ref n) => {
                if let Some(a) = labels.get(&n) {
                    Some(Value::Immediate(a.clone()))
                } else {
                    None
                }
            }
            Value::ProgramCounter => Some(Value::Immediate(Number::from(pc + 1))),
            _ => Some(self.clone()),
        }
    }
}

#[derive(Clone)]
pub enum Address {
    Immediate(Number),
    Register(Number),
    ProgramCounter,
    Label(String),
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Address::Immediate(ref n) => write!(f, "{}", n),
            Address::Register(ref n) => write!(f, "[{}]", n),
            Address::Label(ref n) => write!(f, "{}", n),
            Address::ProgramCounter => write!(f, "pc"),
        }
    }
}

impl Address {
    fn solve(&self, labels: &HashMap<&String, Number>, pc: usize) -> Option<Address> {
        match self {
            Address::Label(ref n) => {
                if let Some(a) = labels.get(&n) {
                    Some(Address::Immediate(a.clone()))
                } else {
                    None
                }
            }
            Address::ProgramCounter => Some(Address::Immediate(Number::from(pc + 1))),
            _ => Some(self.clone()),
        }
    }
}

#[derive(Clone)]
pub enum Statement {
    Incr(Index, Value),
    Decr(Index, Address, Value),
    Save(Index, Value),
    Putc(Value),
    Putn(Value),
    Halt,
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Statement::Incr(ref i, ref v) => write!(f, "incr {}, {}", i, v),
            Statement::Decr(ref i, ref a, ref v) => write!(f, "decr {}, {}, {}", i, a, v),
            Statement::Save(ref i, ref v) => write!(f, "save {}, {}", i, v),
            Statement::Putc(ref v) => write!(f, "putc {}", v),
            Statement::Putn(ref v) => write!(f, "putn {}", v),
            Statement::Halt => write!(f, "halt"),
        }
    }
}

pub struct Line {
    label: Option<String>,
    statement: Statement,
}

impl Line {
    pub fn new(label: Option<String>, statement: Statement) -> Line {
        Line { label, statement }
    }
}

pub struct Ast(pub Vec<Line>);

use std::ops::{Deref, DerefMut};

impl Deref for Ast {
    type Target = Vec<Line>;
    fn deref(&self) -> &Vec<Line> {
        &self.0
    }
}

impl fmt::Display for Ast {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for x in self.iter() {
            if let Err(e) = write!(
                f,
                "{}\t{}\n",
                match &x.label {
                    Some(label) => &label[..],
                    None => "",
                },
                x.statement
            ) {
                return Err(e);
            }
        }
        Ok(())
    }
}

use std::collections::HashMap;

impl<'a> Ast {
    fn collect_labels(&'a self) -> HashMap<&'a String, Number> {
        let mut h = HashMap::new();
        for (
            i,
            &Line {
                ref label,
                statement: _,
            },
        ) in self.iter().enumerate()
        {
            if let Some(ref label) = label {
                h.insert(label, Number::from(i));
            }
        }
        h
    }
}

pub struct Program(Vec<Statement>);

impl Deref for Program {
    type Target = Vec<Statement>;
    fn deref(&self) -> &Vec<Statement> {
        &self.0
    }
}

impl DerefMut for Program {
    fn deref_mut(&mut self) -> &mut Vec<Statement> {
        &mut self.0
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for x in self.iter() {
            if let Err(e) = write!(f, "{}\n", x) {
                return Err(e);
            }
        }
        Ok(())
    }
}

impl Program {
    pub fn new(ast: Ast) -> Option<Program> {
        let labels = ast.collect_labels();
        let mut program = Vec::<Statement>::new();
        for (pc, x) in ast.iter().enumerate() {
            match &x.statement {
                Statement::Decr(index, address, value) => program.push(Statement::Decr(
                    index.clone(),
                    address.solve(&labels, pc)?.clone(),
                    value.solve(&labels, pc)?,
                )),
                Statement::Incr(index, value) => {
                    program.push(Statement::Incr(index.clone(), value.solve(&labels, pc)?))
                }
                Statement::Save(index, value) => {
                    program.push(Statement::Save(index.clone(), value.solve(&labels, pc)?))
                }
                Statement::Putc(value) => program.push(Statement::Putc(value.solve(&labels, pc)?)),
                Statement::Putn(value) => program.push(Statement::Putn(value.solve(&labels, pc)?)),
                Statement::Halt => program.push(Statement::Halt),
            }
        }
        Some(Program(program))
    }
}
