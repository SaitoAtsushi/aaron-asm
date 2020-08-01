extern crate num_bigint;
extern crate num_traits;
pub type Number = num_bigint::BigInt;

use std::fmt;
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
    fn solve(&self, labels: &HashMap<&String, Number>) -> Option<Value> {
        if let Value::Label(ref n) = self {
            if let Some(a) = labels.get(&n) {
                Some(Value::Immediate(a.clone()))
            } else {
                None
            }
        } else {
            Some(self.clone())
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
    fn solve(&self, labels: &HashMap<&String, Number>) -> Option<Address> {
        if let Address::Label(ref n) = self {
            if let Some(a) = labels.get(&n) {
                Some(Address::Immediate(a.clone()))
            } else {
                None
            }
        } else {
            Some(self.clone())
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

// impl fmt::Display for Line {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(
//             f,
//             "{}\t{}",
//             match &self.label {
//                 Some(ref label) => &label[..],
//                 None => "",
//             },
//             self.statement
//         )
//     }
// }

impl Line {
    pub fn new(label: Option<String>, statement: Statement) -> Line {
        Line { label, statement }
    }
}

struct Ast(pub Vec<Line>);

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
    fn new(ast: Ast) -> Option<Program> {
        let labels = ast.collect_labels();
        let mut program = Vec::<Statement>::new();
        for x in ast.iter() {
            match &x.statement {
                Statement::Decr(index, address, value) => program.push(Statement::Decr(
                    index.clone(),
                    address.solve(&labels)?.clone(),
                    value.solve(&labels)?,
                )),
                Statement::Incr(index, value) => {
                    program.push(Statement::Incr(index.clone(), value.solve(&labels)?))
                }
                Statement::Save(index, value) => {
                    program.push(Statement::Save(index.clone(), value.solve(&labels)?))
                }
                Statement::Putc(value) => {
                    program.push(Statement::Putc(value.solve(&labels)?))
                }
                Statement::Putn(value) => {
                    program.push(Statement::Putn(value.solve(&labels)?))
                }
                Statement::Halt => program.push(Statement::Halt),
            }
        }
        Some(Program(program))
    }
}

extern crate peg;

peg::parser! {
    grammar aaron_parser() for str {
        rule ident() -> String = n:$(['a'..='z' | 'A'..='Z']+ ['a'..='z' | 'A'..='Z' | '0'..='9']*) { String::from(n) }
        rule many_space() = [' ' | '\t']*
        rule separator() = many_space() "," many_space()
        rule comment() ->() = many_space() (";" [n if n!='\n'] *)?
        rule number() -> Number = n:$((['-']? ['1'..='9'] ['0'..='9']*) / (['0'])) { n.parse().unwrap() }
        rule index() -> Index
            = "[" many_space() n:number() many_space() "]" { Index::Indirect(n) }
            / n:number() { Index::Direct(n)}
        rule value() -> Value
            = "[[" many_space() n:number() many_space() "]]" {Value::Pointer(n)}
            / "[" many_space() n:number() many_space() "]" {Value::Register(n)}
            / i:ident() {if i=="pc" {Value::ProgramCounter} else {Value::Label(i)}}
            / n:number() {Value::Immediate(n)}
        rule address() -> Address
            = "[" many_space() n:number() many_space() "]" {Address::Register(n)}
            / "pc" {Address::ProgramCounter}
            / n:ident() {Address::Label(n)}
            / n:number() {Address::Immediate(n)}
        rule command() -> Statement
            = "incr" many_space() i:index() v:(separator() v:value(){v})? {
                match v {
                    Some(v) => Statement::Incr(i, v),
                    None => Statement::Incr(i, Value::Immediate(Number::from(1)))
                }
            }
            / "decr" many_space() i:index() separator() a:address() v:(separator() v:value(){v})? {
                match v {
                    Some(v) => Statement::Decr(i, a, v),
                    None => Statement::Decr(i, a, Value::Immediate(Number::from(1)))
                }
            }
            / "save" many_space() i:index() separator() v:value() { Statement::Save(i,v) }
            / "putc" many_space() v:value() { Statement::Putc(v)}
            / "putn" many_space() v:value() { Statement::Putn(v)}
            / "halt" {Statement::Halt}
        rule line() -> Line
            = (comment() "\n")? label:ident()? many_space() c:command() comment() {Line::new(label, c)}
        pub rule parse() -> Program = v:line() ** "\n" (comment() ** "\n")? {? Program::new(Ast(v)).ok_or("unknown label") }
    }
}

use std::str::FromStr;

impl FromStr for Program {
    type Err = String;

    fn from_str(source: &str) -> std::result::Result<Program, String> {
        let ast = aaron_parser::parse(source);
        match ast {
            Ok(ast) => Ok(ast),
            Err(a) => Err(format!("parse error on Line {}", a.location)),
        }
    }
}
