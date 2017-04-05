extern crate regex;

use regex::*;
use std::{env, io};
use std::collections::HashMap;
use std::str::FromStr;

macro_rules! printerr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

const VALID_TOKEN_RE: &'static str = "\\+|-|\\*|/|%|!|\\(|\\)|\"|!=|<=|>=|<|>|==";
const WHITESPACE_NL_RE: &'static str = " +|\\n+";
const NUM_RE: &'static str = r"-?[0-9]+";
const SYM_RE: &'static str = "\"[.]\"";

/*
    TODO 4/5
    --> debug parenthetical expressions
    ------> figure out where to call next() and where to call peek()
    --> square-bracket all code?
*/

/********************************* definitions ***************************************************/

enum Val {
    Num { value: i64 },
    Sym { value: String },
    Bool { value: bool },
    Nil,
}

impl Clone for Val {
    fn clone(&self) -> Val {
        match *self {
            Val::Num { ref value } => Val::Num { value: value.clone() },
            Val::Sym { ref value } => Val::Sym { value: value.clone() },
            Val::Bool { ref value } => Val::Bool { value: value.clone() },
            Val::Nil => Val::Nil,
        }
    }
}

type Env = HashMap<String, Val>;
const PROMPT: &'static str = ">>";

enum BinaryOp { Add, Sub, Mult, Div, Mod, Lt, Gt, Lte, Gte, Eq, Neq, And, Or }
enum UnaryOp { Negate, Inverse }
enum Exp {
    Literal { value: Val },
    //VarName { name: String },
    Binary { operator: BinaryOp, left: Box<Exp>, right: Box<Exp>},
    Unary { operator: UnaryOp, operand: Box<Exp> },
    //If { e1: Box<Exp>, e2: Box<Exp>, e3: Box<Exp> },
    //Let { name: String, e: Box<Exp> },
    Empty,
}

/************************************* parser ****************************************************/

type Tokens<'a> = std::iter::Peekable<std::slice::Iter<'a, &'a str>>;

// TODO: future goal: split lexical analysis and parsing steps more clearly
fn parse(raw_code: &str) -> Vec<Exp> {
    let mut exps = Vec::new();
    let lines = raw_code.trim().split(';'); // TODO handle SYM

    for line in lines {
        // perform lexical analysis
        let wsnl_re = Regex::new(WHITESPACE_NL_RE).unwrap();
        let tokens_re = Regex::new(VALID_TOKEN_RE).unwrap();
        let temp = tokens_re.replace_all(line, |caps: &Captures| format!(" {} ", &caps[0]));

        // perform parsing
        let ts: Vec<&str> = wsnl_re.split(temp.trim()).collect();
        match parse_line(&mut ts.iter().peekable()) {
            Ok(exp) => exps.push(exp),
            Err(err) => printerr!("syntax error: {}", err.to_string()),
        }
    }
    exps
}

fn parse_line<'a>(ts: &mut Tokens<'a>) -> Result<Exp, String> {
    let e = try!(expr(ts));
    if let Some(token) = ts.peek() {
        Err(format!("unknown token \'{}\'", token))
    } else {
        Ok(e)
    }
}

fn expr<'a>(ts: &mut Tokens<'a>) -> Result<Exp, String> {
    equality(ts)
}

fn equality<'a>(ts: &mut Tokens<'a>) -> Result<Exp, String> {
    let mut e = try!(comparison(ts));

    while let Some(op) = ts.clone().peek() {
        match op.as_ref() {
            "==" => {
                ts.next();
                e = Exp::Binary {
                    operator: BinaryOp::Eq,
                    left: Box::new(e),
                    right: Box::new(try!(comparison(ts)))
                }
            },
            "!=" => {
                ts.next();
                e = Exp::Binary {
                    operator: BinaryOp::Neq,
                    left: Box::new(e),
                    right: Box::new(try!(comparison(ts)))
                }
            },
            _ => break,
        }
    }
    Ok(e)
}

fn comparison<'a>(ts: &mut Tokens<'a>) -> Result<Exp, String> {
    let mut e = try!(term(ts));
    while let Some(op) = ts.clone().peek() {
        match op.as_ref() {
            ">" => {
                ts.next();
                e = Exp::Binary {
                    operator: BinaryOp::Gt,
                    left: Box::new(e),
                    right: Box::new(try!(term(ts)))
                }
            },
            "<" => {
                ts.next();
                e = Exp::Binary {
                    operator: BinaryOp::Lt,
                    left: Box::new(e),
                    right: Box::new(try!(term(ts)))
                }
            },
            ">=" => {
                ts.next();
                e = Exp::Binary {
                    operator: BinaryOp::Gte,
                    left: Box::new(e),
                    right: Box::new(try!(term(ts)))
                }
            },
            "<=" => {
                ts.next();
                e = Exp::Binary {
                    operator: BinaryOp::Lte,
                    left: Box::new(e),
                    right: Box::new(try!(term(ts)))
                }
            },
            _ => break,
        }
    }
    Ok(e)
}

fn term<'a>(ts: &mut Tokens<'a>) -> Result<Exp, String> {
    let mut e = try!(factor(ts));

    // TODO: NOT a permanent solution
    // this clone is highly inefficient and needs to be changed
    while let Some(op) = ts.clone().peek() {
        match op.as_ref() {
            "-" => {
                ts.next();
                e = Exp::Binary {
                    operator: BinaryOp::Sub,
                    left: Box::new(e),
                    right: Box::new(try!(factor(ts)))
                }
            },
            "+" => {
                ts.next();
                e = Exp::Binary {
                    operator: BinaryOp::Add,
                    left: Box::new(e),
                    right: Box::new(try!(factor(ts)))
                }
            },
            _ => break,
        };
    }
    Ok(e)
}

fn factor<'a>(ts: &mut Tokens<'a>) -> Result<Exp, String> {
    let mut e = try!(unary(ts));

    while let Some(op) = ts.clone().peek() {
        match op.as_ref() {
            "/" => {
                ts.next();
                e = Exp::Binary {
                    operator: BinaryOp::Div,
                    left: Box::new(e),
                    right: Box::new(try!(unary(ts)))
                }
            },
            "*" => {
                ts.next();
                e = Exp::Binary {
                    operator: BinaryOp::Mult,
                    left: Box::new(e),
                    right: Box::new(try!(unary(ts)))
                }
            },
            "%" => {
                ts.next();
                e = Exp::Binary {
                    operator: BinaryOp::Mod,
                    left: Box::new(e),
                    right: Box::new(try!(unary(ts)))
                }
            },
            _ => break,
        };
    }
    Ok(e)
}

fn unary<'a>(ts: &mut Tokens<'a>) -> Result<Exp, String> {
    let op = match ts.peek() {
        Some(op) => op.clone(),
        None => return Err("expected unary operator or primary token".to_string()),
    };

    match op.as_ref() as &str {
        "!" => {
            Ok(Exp::Unary { operator: UnaryOp::Inverse, operand: Box::new(try!(unary(ts))) })
        },
        "-" => {
            Ok(Exp::Unary { operator: UnaryOp::Negate, operand: Box::new(try!(unary(ts))) })
        },
        _ => primary(ts),
    }
}

fn primary<'a>(ts: &mut Tokens<'a>) -> Result<Exp, String> {
    let num_re = Regex::new(NUM_RE).unwrap();
    let sym_re = Regex::new(SYM_RE).unwrap();

    // don't need to match for this because coming directly from unary
    let next = ts.next().unwrap().to_string();

    match next.as_ref() {
        "false" => { Ok(Exp::Literal { value: Val::Bool { value: false } }) },
        "true" => { Ok(Exp::Literal { value: Val::Bool { value: true } }) },
        "nil" => { Ok(Exp::Literal { value: Val::Nil }) },
        "(" => {
            let e = expr(ts);
            match ts.next() {
                Some(ch) if ch.to_string() == ")".to_string() => e,
                Some(_) => Err("bug in parser".to_string()),
                None => Err("missing closing parenthesis".to_string()),
            }
        },
        _ => {
            if num_re.is_match(&next) {
                Ok(Exp::Literal { value: Val::Num { value: i64::from_str(&next).unwrap() } })
            } /* else if sym_re.is_match(&next) {
                Ok(Exp::Literal { value: Val::Sym { value: next }})
            }*/ else {
                Err(format!("expected primary token, got {}", next))
            }
        }
    }
}

/*********************************** evaluator ***************************************************/
/*
fn eval_exps(exps: Vec<Exp>, env: &mut Env) {
    for e in exps {
        eval(e, env);
    }
}

fn eval(e: Exp, env: &mut Env) -> Val {
/*    match e {
        Exp::Literal { value } => value,
        Exp::VarName { name } => {
            match env.get(&name) {
                Some(value) => (*value).clone(),
                None => {
                    println!("error: variable \"{}\" not bound", name);
                    Val::Nil {}
                }
            }
        },
        Exp::Let { name, e } => {
            let v = eval(*e, env);
            env.insert(name, v);
            Val::Bool { value: true }
        },
        Exp::Empty {} => Val::Nil {},
    }*/
    Val::Nil {}
}
*/
/*************************************************************************************************/
use std::io::prelude::*;
fn main() {
    /* initialization
            + initialize environment and load environment variables
            + set default PATH
            + load .rushrc
            + set current directory to current user's home
    */
    let mut env: Env = HashMap::new();
    let stdin = io::stdin();


    loop {
        print!("{} ", PROMPT);
        io::Write::flush(&mut io::stdout()).expect("flush failed!");

        let mut code = String::new();
        match stdin.lock().read_line(&mut code) {
            Ok(_n) => {
                code.pop();
                let exps = parse(&code);
                //eval_exps(exps, &mut env);
            }
            Err(error) => printerr!("error: {}", error),
        }
    }
}
