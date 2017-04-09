extern crate regex;

#[macro_use]
extern crate lazy_static;

use regex::*;
use std::{env, io};
use std::collections::HashMap;
use std::str::FromStr;
use std::process::Command;
macro_rules! printerr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

const VALID_TOKEN_RE: &'static str =
    "\\+|-|\\*|/|%|!(?:=)|\\(|\\)|\"|!=|<=|>=|<|>|==|&&|\\|\\||if|then|else";
const VALID_EXP_START: &'static str = "if|then|else|:|!|-";
const WHITESPACE_NL_RE: &'static str = " +|\\n+";
const NUM_RE: &'static str = r"-?[0-9]+";
const SYM_RE: &'static str = "\"[.]\"";

/*
    TODO 4/5
    --> square-bracket all code?
    --> basic shell functionality
*/

/********************************* definitions ***************************************************/

enum Val {
    Num(i64),
    Sym(String),
    Bool(bool),
    Nil,
    Command(String, Vec<Box<Val>>)
}

impl Clone for Val {
    fn clone(&self) -> Val {
        match *self {
            Val::Num(ref value) => Val::Num(value.clone()),
            Val::Sym(ref value) => Val::Sym(value.clone()),
            Val::Bool(ref value) => Val::Bool(value.clone()),
            Val::Command(ref cmd, ref args) => Val::Command(cmd.clone(), args.clone()),
            Val::Nil => Val::Nil
        }
    }
}

type Env = HashMap<String, Val>;
lazy_static! {
    static ref ENV: Env = {
        HashMap::new()
    };
}

const PROMPT: &'static str = "->";

enum BinaryOp { Add, Sub, Mult, Div, Mod, Lt, Gt, Lte, Gte, Eq, Neq, And, Or }
enum UnaryOp { Negate, Inverse }
enum Exp {
    Literal(Val),
    Binary(Box<Exp>, BinaryOp, Box<Exp>),
    Unary(UnaryOp, Box<Exp>),
    If(Box<Exp>, Box<Exp>, Box<Exp>),
    Command(String, Vec<Exp>),
    Empty
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
    toplevel(ts)
}

/*************************************** EXPERIMENTAL ********************************************/

fn toplevel<'a>(ts: &mut Tokens<'a>) -> Result<Exp, String> {
    let token = match ts.peek() {
        Some(op) => op.clone(),
        None => return Ok(Exp::Empty),
    };
    if Regex::new(VALID_EXP_START).unwrap().is_match(token) {
        expr(ts)
    } else {
        let mut args = Vec::new();
        while let Some(op) = ts.next() {
            match op.as_ref() {
                "[" => {
                    args.push(try!(expr(ts)));
                    match ts.next() {
                        Some(ch) if ch.to_string() != "]".to_string() => return Err("expected closing bracket".to_string()),
                        _ => return Err("missing closing parenthesis".to_string()),
                    };
                },
                arg => args.push(Exp::Literal(Val::Sym(arg.to_string())))
            }
        }
        Ok(Exp::Command(token.to_string(), args))
    }
}

fn expr<'a>(ts: &mut Tokens<'a>) -> Result<Exp, String> {
    if let Some(token) = ts.clone().peek() {
        if token.to_string() == "if".to_string() {
            ts.next();
            let cond = try!(expr(ts));
            if let Some(token) = ts.clone().peek() {
                if token.to_string() != "then".to_string() {
                    return Err(format!("expected token \'then\', got {}", token));
                }
            }
            ts.next();
            let branch1 = try!(expr(ts));

            if let Some(token) = ts.clone().next() {
                if token.to_string() != "else".to_string() {
                    return Err(format!("expected token \'then\', got {}", token));
                }
            }
            ts.next();
            let branch2 = try!(expr(ts));
            Ok(Exp::If(Box::new(cond), Box::new(branch1), Box::new(branch2)))
        } else {
            equality(ts)
        }
    } else{
        Ok(Exp::Empty)
    }
}

fn equality<'a>(ts: &mut Tokens<'a>) -> Result<Exp, String> {
    let mut e = try!(comparison(ts));

    while let Some(op) = ts.clone().peek() {
        match op.as_ref() {
            "==" => {
                ts.next();
                e = Exp::Binary(Box::new(e), BinaryOp::Eq, Box::new(try!(comparison(ts))))
            },
            "!=" => {
                ts.next();
                e = Exp::Binary(Box::new(e), BinaryOp::Neq, Box::new(try!(comparison(ts))))
            },
            _ => break,
        }
    }
    Ok(e)
}

fn comparison<'a>(ts: &mut Tokens<'a>) -> Result<Exp, String> {
    let mut e = try!(boolop(ts));
    while let Some(op) = ts.clone().peek() {
        match op.as_ref() {
            ">" => {
                ts.next();
                e = Exp::Binary(Box::new(e), BinaryOp::Gt, Box::new(try!(boolop(ts))))
            },
            "<" => {
                ts.next();
                e = Exp::Binary(Box::new(e), BinaryOp::Lt, Box::new(try!(boolop(ts))))
            },
            ">=" => {
                ts.next();
                e = Exp::Binary(Box::new(e), BinaryOp::Gte, Box::new(try!(boolop(ts))))
            },
            "<=" => {
                ts.next();
                e = Exp::Binary(Box::new(e), BinaryOp::Lte, Box::new(try!(boolop(ts))))
            },
            _ => break,
        }
    }
    Ok(e)
}

fn boolop<'a>(ts: &mut Tokens<'a>) -> Result<Exp, String> {
    let mut e = try!(term(ts));
    while let Some(op) = ts.clone().peek() {
        match op.as_ref() {
            "&&" | "and" => {
                ts.next();
                e = Exp::Binary(Box::new(e), BinaryOp::And, Box::new(try!(term(ts))))
            },
            "||" | "or" => {
                ts.next();
                e = Exp::Binary(Box::new(e), BinaryOp::Or, Box::new(try!(term(ts))))
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
                e = Exp::Binary(Box::new(e), BinaryOp::Sub, Box::new(try!(factor(ts))))
            },
            "+" => {
                ts.next();
                e = Exp::Binary(Box::new(e), BinaryOp::Add, Box::new(try!(factor(ts))))
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
                e = Exp::Binary(Box::new(e), BinaryOp::Div, Box::new(try!(unary(ts))))
            },
            "*" => {
                ts.next();
                e = Exp::Binary(Box::new(e), BinaryOp::Mult, Box::new(try!(unary(ts))))
            },
            "%" => {
                ts.next();
                e = Exp::Binary(Box::new(e), BinaryOp::Mod, Box::new(try!(unary(ts))))
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
            ts.next();
            Ok(Exp::Unary(UnaryOp::Inverse, Box::new(try!(unary(ts)))))
        },
        "-" => {
            ts.next();
            Ok(Exp::Unary(UnaryOp::Negate, Box::new(try!(unary(ts)))))
        },
        _ => primary(ts),
    }
}

fn primary<'a>(ts: &mut Tokens<'a>) -> Result<Exp, String> {
    let num_re = Regex::new(NUM_RE).unwrap();
    let _sym_re = Regex::new(SYM_RE).unwrap();

    // don't need to match for this because coming directly from unary
    let next = ts.next().unwrap().to_string();

    match next.as_ref() {
        "false" => Ok(Exp::Literal(Val::Bool(false))),
        "true" => Ok(Exp::Literal(Val::Bool(true))),
        "nil" => Ok(Exp::Literal(Val::Nil)),
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
                Ok(Exp::Literal(Val::Num(i64::from_str(&next).unwrap())))
            } /* else if sym_re.is_match(&next) {
                Ok(Exp::Literal { value: Val::Sym { value: next }})
            }*/ else {
                Err(format!("expected primary token, got {}", next))
            }
        }
    }
}

/*********************************** evaluator ***************************************************/

fn print_value(v: Val) {
    match v {
        Val::Num(value) => println!("{} : Num", value),
        Val::Bool(value) => println!("{} : Bool", value),
        Val::Sym(value) => println!("{} : Sym", value ),
        Val::Nil => println!("nil : Nil"),
        Val::Command(_, _) => printerr!("bug in evaluator")
    };
}

fn eval_exps(exps: Vec<Exp>) {
    for e in exps {
        match eval(e) {
            Ok(v) => print_value(v),
            Err(err) => printerr!("error evaluating expression: {}", err)
        };
    }
}

fn expect_num(v: Val) -> Result<i64, String> {
    match v {
        Val::Num(value) => Ok(value),
        Val::Bool(_) => Err("expected num, got bool".to_string()),
        Val::Sym(_) => Err("expected num, got sym".to_string()),
        Val::Nil => Err("expected num, got nil".to_string()),
        Val::Command(_, _) => Err("bug in evaluator".to_string())
    }
}

fn expect_bool(v: Val) -> Result<bool, String> {
    match v {
        Val::Num(_) => Err("expected bool, got num".to_string()),
        Val::Bool(value) => Ok(value),
        Val::Sym(_) => Err("expected bool, got sym".to_string()),
        Val::Nil => Err("expected bool, got nil".to_string()),
        Val::Command(_, _) => Err("bug in evaluator".to_string())
    }
}

fn eval_binop(l: Exp, op: BinaryOp, r: Exp) -> Result<Val, String> {
    let lval = try!(eval(l));
    let rval = try!(eval(r));

    match op {
        BinaryOp::Add => Ok(Val::Num(try!(expect_num(lval)) + try!(expect_num(rval)))),
        BinaryOp::Sub => Ok(Val::Num(try!(expect_num(lval)) - try!(expect_num(rval)))),
        BinaryOp::Mult => Ok(Val::Num(try!(expect_num(lval)) * try!(expect_num(rval)))),
        BinaryOp::Div => Ok(Val::Num(try!(expect_num(lval)) / try!(expect_num(rval)))),
        BinaryOp::Mod => Ok(Val::Num(try!(expect_num(lval)) % try!(expect_num(rval)))),
        BinaryOp::Lt => Ok(Val::Bool(try!(expect_num(lval)) < try!(expect_num(rval)))),
        BinaryOp::Lte => Ok(Val::Bool(try!(expect_num(lval)) <= try!(expect_num(rval)))),
        BinaryOp::Gt => Ok(Val::Bool(try!(expect_num(lval)) > try!(expect_num(rval)))),
        BinaryOp::Gte => Ok(Val::Bool(try!(expect_num(lval)) >= try!(expect_num(rval)))),
        BinaryOp::Eq => Ok(Val::Bool(try!(expect_num(lval)) == try!(expect_num(rval)))),
        BinaryOp::Neq => Ok(Val::Bool(try!(expect_num(lval)) != try!(expect_num(rval)))),
        BinaryOp::And => Ok(Val::Bool(try!(expect_bool(lval)) && try!(expect_bool(rval)))),
        BinaryOp::Or => Ok(Val::Bool(try!(expect_bool(lval)) || try!(expect_bool(rval))))
    }
}

fn eval_unop(operator: UnaryOp, operand: Exp) -> Result<Val, String> {
    let val = try!(eval(operand));
    match operator {
        UnaryOp::Negate => Ok(Val::Num(try!(expect_num(val)) * -1 )),
        UnaryOp::Inverse => Ok(Val::Num(try!(expect_num(val)) * -1 )),
    }
}

fn eval_if(cond: Exp, branch1: Exp, branch2: Exp) -> Result<Val, String> {
    if try!(expect_bool(try!(eval(cond)))) {
        eval(branch1)
    } else {
        eval(branch2)
    }
}

fn execute_command(cmd: &str, args: Vec<String>) -> Result<Val, String> {
    println!("cmd: |{}|", cmd);
    let output = Command::new(cmd)
                     .args(args)
                     .output()
                     .expect(&format!("command \'{}\'", cmd));
    Ok(Val::Sym(String::from_utf8(output.stderr).expect("unable to convert bytes to string")))
}

fn eval_command(cmd: &str, args: Vec<Exp>) -> Result<Val, String> {
    let mut arg_vals = Vec::new();
    for arg in args {
        let arg_val = match try!(eval(arg)) {
            Val::Num(v) => v.to_string(),
            Val::Bool(v) => v.to_string(),
            Val::Sym(v) => v,
            Val::Nil => "".to_string(),
            Val::Command(_cmd, _args) => return Err("not sure what I should do here".to_string())
        };
        arg_vals.push(arg_val);
    }

    execute_command(cmd, arg_vals)
}

fn eval(e: Exp) -> Result<Val, String> {
    match e {
        Exp::Literal(value) => Ok(value),
        Exp::Binary(left, op, right) => eval_binop(*left, op, *right),
        Exp::Unary(operator, operand) => eval_unop(operator, *operand),
        Exp::If(cond, branch1, branch2) => eval_if(*cond, *branch1, *branch2),
        Exp::Command(cmd, args) => eval_command(&cmd, args),
        Exp::Empty => Ok(Val::Nil)
    }
}

/*************************************************************************************************/
use std::io::prelude::*;
fn main() {
    /* initialization
            + set default PATH
            + load .rushrc
            + set current directory to current user's home
    */
    let stdin = io::stdin();

    loop {
        print!("{} ", PROMPT);
        io::Write::flush(&mut io::stdout()).expect("flush failed!");

        let mut code = String::new();
        let mut num_parens = 0;
        let mut num_brackets = 0;

        match stdin.lock().read_line(&mut code) {
            Ok(_n) => {
                code.pop();
                let exps = parse(&code);
                eval_exps(exps);
            }
            Err(error) => printerr!("error: {}", error),
        }
    }
}
