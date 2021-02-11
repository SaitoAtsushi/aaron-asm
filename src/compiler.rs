use crate::syntax_tree::*;

#[derive(Debug)]
pub enum ParseError {
    InvalidLabel,
    InvalidIdentifier,
    LabelOnly,
    UnknownMnemonic,
    UnclosedBracket,
    ExpectInteger,
    ExpectValue,
    ExtraZero,
    ExtraOperand,
    TooFewArguments,
    ExpectAddress,
    EndOfProgram,
}

type ParseResult<'a, T> = std::result::Result<(T, &'a str), ParseError>;

fn is_space(ch: char) -> bool {
    ch == ' ' || ch == '\t'
}

fn parse_one(input: &str, predicate: impl Fn(char) -> bool) -> Option<(char, &str)> {
    let mut iter = input.chars();
    iter.next().and_then(|ch| {
        if predicate(ch) {
            Some((ch, iter.as_str()))
        } else {
            None
        }
    })
}

fn parse_while(input: &str, predicate: impl Fn(char) -> bool) -> (&str, &str) {
    if let Some(pos) = input.find(|ch| !predicate(ch)) {
        input.split_at(pos)
    } else {
        let len = input.len();
        (input, &input[len..len])
    }
}

fn parse_skip(input: &str, predicate: impl Fn(char) -> bool) -> &str {
    if let Some(pos) = input.find(|ch| !predicate(ch)) {
        &input[pos..]
    } else {
        &input[input.len()..]
    }
}

fn parse_skip_until(input: &str, predicate: impl Fn(char) -> bool) -> &str {
    if let Some(pos) = input.find(|ch| predicate(ch)) {
        let mut iter = input[pos..].chars();
        iter.next();
        iter.as_str()
    } else {
        &input[input.len()..]
    }
}

fn skip_space(input: &str) -> &str {
    parse_skip(input, |ch| ch == ' ' || ch == '\t')
}

fn skip_comment(input: &str) -> &str {
    parse_skip_until(input, |ch| ch == '\n')
}

fn parse_label(input: &str) -> ParseResult<Option<String>> {
    match parse_one(input, |_| true) {
        Some((ch, _)) if ch.is_ascii_alphabetic() => {
            let (label, rest) = parse_while(input, |ch| ch.is_ascii_alphanumeric());
            Ok((Some(String::from_str(label).unwrap()), rest))
        }
        Some((ch, _)) if is_space(ch) || ch == ';' || ch == '\r' || ch == '\n' => Ok((None, input)),
        Some(_) => Err(ParseError::InvalidLabel),
        None => Ok((None, input)),
    }
}

fn parse_identifier(input: &str) -> ParseResult<String> {
    let _ = parse_one(input, |ch| ch.is_ascii_alphabetic()).ok_or(ParseError::InvalidIdentifier);
    let (label, rest) = parse_while(input, |ch| ch.is_ascii_alphanumeric());
    Ok((String::from_str(label).unwrap(), rest))
}

enum Mnemonic {
    Incr,
    Decr,
    Save,
    Putc,
    Putn,
    Halt,
}

fn parse_mnemonic(input: &str) -> ParseResult<Mnemonic> {
    let (mnemonic, rest) = parse_while(input, |ch| ch.is_ascii_alphanumeric());
    Ok((
        match mnemonic {
            "incr" => Mnemonic::Incr,
            "decr" => Mnemonic::Decr,
            "save" => Mnemonic::Save,
            "putc" => Mnemonic::Putc,
            "putn" => Mnemonic::Putn,
            "halt" => Mnemonic::Halt,
            _ => Err(ParseError::UnknownMnemonic)?,
        },
        rest,
    ))
}

fn skip_extra_field(input: &str) -> std::result::Result<&str, ParseError> {
    let rest = skip_space(input);
    match rest.chars().next() {
        Some(ch) if ch == ';' || ch == '\n' || ch == '\r' => Ok(skip_comment(rest)),
        Some(_) => Err(ParseError::ExtraOperand),
        None => Ok(rest),
    }
}

fn parse_operand_separator(input: &str) -> std::result::Result<&str, ParseError> {
    let rest = skip_space(input);
    let (_, rest) = parse_one(rest, |ch| ch == ',').ok_or(ParseError::TooFewArguments)?;
    let rest = skip_space(rest);
    Ok(rest)
}

fn parse_incr_operand(input: &str) -> ParseResult<Statement> {
    let (index, rest) = parse_index(input)?;
    match parse_operand_separator(rest) {
        Err(_) => Ok((
            Statement::Incr(index, Value::Immediate(Number::from(1))),
            rest,
        )),
        Ok(rest) => {
            let (value, rest) = parse_value(rest)?;
            let rest = skip_space(rest);
            Ok((Statement::Incr(index, value), skip_extra_field(rest)?))
        }
    }
}

fn parse_decr_operand(input: &str) -> ParseResult<Statement> {
    let (index, rest) = parse_index(input)?;
    let rest = parse_operand_separator(rest)?;
    let (address, rest) = parse_address(rest)?;
    match parse_operand_separator(rest) {
        Err(_) => Ok((
            Statement::Decr(index, address, Value::Immediate(Number::from(1))),
            skip_extra_field(rest)?,
        )),
        Ok(rest) => {
            let (value, rest) = parse_value(rest)?;
            Ok((
                Statement::Decr(index, address, value),
                skip_extra_field(rest)?,
            ))
        }
    }
}

fn parse_save_operand(input: &str) -> ParseResult<Statement> {
    let (index, rest) = parse_index(input)?;
    let rest = parse_operand_separator(rest)?;
    let (value, rest) = parse_value(rest)?;
    let rest = skip_extra_field(rest)?;
    Ok((Statement::Save(index, value), rest))
}

fn parse_putc_operand(input: &str) -> ParseResult<Statement> {
    let (value, rest) = parse_value(input)?;
    let rest = skip_extra_field(rest)?;
    Ok((Statement::Putc(value), rest))
}

fn parse_putn_operand(input: &str) -> ParseResult<Statement> {
    let (value, rest) = parse_value(input)?;
    let rest = skip_extra_field(rest)?;
    Ok((Statement::Putn(value), rest))
}

fn parse_halt_operand(input: &str) -> ParseResult<Statement> {
    let rest = skip_extra_field(input)?;
    Ok((Statement::Halt, rest))
}

fn parse_command(input: &str) -> ParseResult<Statement> {
    let (mnemonic, rest) = parse_mnemonic(input)?;
    let rest = skip_space(rest);
    match mnemonic {
        Mnemonic::Incr => parse_incr_operand(rest),
        Mnemonic::Decr => parse_decr_operand(rest),
        Mnemonic::Save => parse_save_operand(rest),
        Mnemonic::Putc => parse_putc_operand(rest),
        Mnemonic::Putn => parse_putn_operand(rest),
        Mnemonic::Halt => parse_halt_operand(rest),
    }
}

fn parse_integer(input: &str) -> ParseResult<Number> {
    let (sign, rest) = parse_one(input, |ch| ch == '-').unwrap_or(('+', input));
    if let Some((_, rest)) = parse_one(rest, |ch| ch == '0') {
        if let Some(_) = parse_one(rest, |ch| ch.is_ascii_digit()) {
            return Err(ParseError::ExtraZero);
        }
    }
    if let Some(_) = parse_one(rest, |ch| ch.is_ascii_digit()) {
        let (num, rest) = parse_while(rest, |ch| ch.is_ascii_digit());
        let mut num: Number = num.parse().unwrap();
        if sign == '-' {
            num = -num
        }
        Ok((num, rest))
    } else {
        Err(ParseError::ExpectInteger)
    }
}

fn parse_index(input: &str) -> ParseResult<Index> {
    if let Some((_, rest)) = parse_one(input, |ch| ch == '[') {
        let rest = skip_space(rest);
        let (num, rest) = parse_integer(rest)?;
        let rest = skip_space(rest);
        if let Some((_, rest)) = parse_one(rest, |ch| ch == ']') {
            Ok((Index::Indirect(num), rest))
        } else {
            Err(ParseError::UnclosedBracket)
        }
    } else {
        let (num, rest) = parse_integer(input)?;
        Ok((Index::Direct(num), rest))
    }
}

fn parse_address(input: &str) -> ParseResult<Address> {
    if let Some((_, rest)) = parse_one(input, |ch| ch == '[') {
        let rest = skip_space(rest);
        let (num, rest) = parse_integer(rest)?;
        let rest = skip_space(rest);
        let (_, rest) = parse_one(rest, |ch| ch == ']').ok_or(ParseError::UnclosedBracket)?;
        Ok((Address::Register(num), rest))
    } else if let Ok((num, rest)) = parse_integer(input) {
        Ok((Address::Immediate(num), rest))
    } else if let Ok((ident, rest)) = parse_identifier(input) {
        if ident == "pc" {
            Ok((Address::ProgramCounter, rest))
        } else {
            Ok((Address::Label(ident), rest))
        }
    } else {
        Err(ParseError::ExpectAddress)
    }
}

fn parse_value(input: &str) -> ParseResult<Value> {
    if let Some((_, rest)) = parse_one(input, |ch| ch == '[') {
        if let Some((_, rest)) = parse_one(rest, |ch| ch == '[') {
            let rest = skip_space(rest);
            let (num, rest) = parse_integer(rest)?;
            let rest = skip_space(rest);
            let (_, rest) = parse_one(rest, |ch| ch == ']').ok_or(ParseError::UnclosedBracket)?;
            let (_, rest) = parse_one(rest, |ch| ch == ']').ok_or(ParseError::UnclosedBracket)?;
            Ok((Value::Pointer(num), rest))
        } else {
            let rest = skip_space(rest);
            let (num, rest) = parse_integer(rest)?;
            let rest = skip_space(rest);
            let (_, rest) = parse_one(rest, |ch| ch == ']').ok_or(ParseError::UnclosedBracket)?;
            Ok((Value::Register(num), rest))
        }
    } else if let Ok((num, rest)) = parse_integer(input) {
        Ok((Value::Immediate(num), rest))
    } else if let Ok((ident, rest)) = parse_identifier(input) {
        if ident == "pc" {
            Ok((Value::ProgramCounter, rest))
        } else {
            Ok((Value::Label(ident), rest))
        }
    } else {
        Err(ParseError::ExpectValue)
    }
}

fn parse_line(input: &str) -> ParseResult<Line> {
    let (label, rest) = parse_label(input)?;
    let rest = skip_space(rest);
    match rest.chars().next() {
        Some(';') | Some('\n') => label.map_or_else(
            || parse_line(skip_comment(rest)),
            |_| Err(ParseError::LabelOnly),
        ),
        Some(_) => {
            let (command, rest) = parse_command(rest)?;
            Ok((Line::new(label, command), rest))
        }
        _ => Err(ParseError::EndOfProgram),
    }
}

fn parse(input: &str) -> std::result::Result<Ast, ParseError> {
    let mut lines = Vec::new();
    let mut input = input;
    let mut count = 0;
    loop {
        match parse_line(input) {
            Ok((line, rest)) => {
                lines.push(line);
                input = rest;
            }
            Err(ParseError::EndOfProgram) => break,
            Err(err) => {
                println!("{}", count);
                Err(err)?
            }
        }
        count += 1;
    }
    Ok(Ast(lines))
}

use std::str::FromStr;

impl FromStr for Program {
    type Err = String;

    fn from_str(source: &str) -> std::result::Result<Program, String> {
        let ast = parse(source);
        match ast {
            Ok(ast) => Ok(Program::new(ast).ok_or("Unknown label")?),
            Err(err) => Err(format!("{:?}", err)),
        }
    }
}
