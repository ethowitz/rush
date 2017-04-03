extern crate regex;

use regex::Regex;
use std::{env, io};
use std::collections::HashMap;
use std::str::FromStr;

// mod trie;

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
    //Command { cmd: String, args: Vec<String> }, // catch-all
    Empty,
}

/************************************* parser ****************************************************/

type Tokens<'a> = std::iter::Peekable<std::slice::Iter<'a, &'a str>>;

// TODO: currently this would only recognize 1 + 2 and not 1+2
//      --> replace important every "token" with " token " to ensure split will catch
//          all the important tokens
fn tokenize<'a>(exp: &'a str) -> Vec<&'a str> {
    let re = Regex::new(r" +|\n+").unwrap();
    re.split(exp).collect()
}

fn parse(raw_code: &str) -> Vec<Exp> {
    let mut exps = Vec::new();

    for line in raw_code.split(';') { // TODO handle strings
        let ts = tokenize(line);
        exps.push(parse_line(&mut ts.iter().peekable()));
    }
    exps
}

fn parse_line<'a>(ts: &mut Tokens<'a>) -> Exp {
    expr(ts)
}

fn expr<'a>(ts: &mut Tokens<'a>) -> Exp {
    equality(ts)
}

fn equality<'a>(ts: &mut Tokens<'a>) -> Exp {
    let mut e = comparison(ts);

    // this is not so idiomaitc, but rust's while let patterns are not expressive enough :(
    loop {
        let op = ts.peek().unwrap().to_string();
        match op.as_ref() {
            "==" => e = Exp::Binary {
                operator: BinaryOp::Eq,
                left: Box::new(e),
                right: Box::new(comparison(ts))
            },
            "!=" => e = Exp::Binary {
                operator: BinaryOp::Neq,
                left: Box::new(e),
                right: Box::new(comparison(ts))
            },
            _ => break,
        };
    }
    e
}

fn comparison<'a>(ts: &mut Tokens<'a>) -> Exp {
    let mut e = term(ts);
    loop {
        let op = ts.peek().unwrap().to_string();
        match op.as_ref() {
            ">" => e = Exp::Binary {
                operator: BinaryOp::Gt,
                left: Box::new(e),
                right: Box::new(term(ts))
            },
            "<" => e = Exp::Binary {
                operator: BinaryOp::Lt,
                left: Box::new(e),
                right: Box::new(term(ts))
            },
            ">=" => e = Exp::Binary {
                operator: BinaryOp::Gte,
                left: Box::new(e),
                right: Box::new(term(ts))
            },
            "<=" => e = Exp::Binary {
                operator: BinaryOp::Lte,
                left: Box::new(e),
                right: Box::new(term(ts))
            },
            _ => break,
        };
    }
    e
}

fn term<'a>(ts: &mut Tokens<'a>) -> Exp {
  let mut e = factor(ts);

  loop {
      let op = ts.peek().unwrap().to_string();
      match op.as_ref() {
          "-" => e = Exp::Binary {
              operator: BinaryOp::Sub,
              left: Box::new(e),
              right: Box::new(term(ts))
          },
          "+" => e = Exp::Binary {
              operator: BinaryOp::Add,
              left: Box::new(e),
              right: Box::new(term(ts))
          },
          _ => break,
      };
  }

  e
}

fn factor<'a>(ts: &mut Tokens<'a>) -> Exp {
  let mut e = unary(ts);

  loop {
      let op = ts.peek().unwrap().to_string();
      match op.as_ref() {
          "/" => e = Exp::Binary {
              operator: BinaryOp::Div,
              left: Box::new(e),
              right: Box::new(term(ts))
          },
          "*" => e = Exp::Binary {
              operator: BinaryOp::Mult,
              left: Box::new(e),
              right: Box::new(term(ts))
          },
          "%" => e = Exp::Binary {
              operator: BinaryOp::Mod,
              left: Box::new(e),
              right: Box::new(term(ts))
          },
          _ => break,
      };
  }

  e
}

fn unary<'a>(ts: &mut Tokens<'a>) -> Exp {
  let op = ts.peek().unwrap().to_string();
  match op.as_ref() {
      "!" => {
          let right = unary(ts);
          Exp::Unary {
              operator: UnaryOp::Inverse,
              operand: Box::new(right)
          }
      },
      "-" => {
          let right = unary(ts);
          Exp::Unary {
              operator: UnaryOp::Negate,
              operand: Box::new(right)
          }
      },
      _ => primary(ts)
  }
}

fn primary<'a>(ts: &mut Tokens<'a>) -> Exp {
    let num_re = Regex::new(r"-?\d\d\d\d\d\d\d\d").unwrap();
    let sym_re = Regex::new("\"[.]\"").unwrap();
    let next = ts.peek().unwrap().to_string();

    match next.as_ref() {
        "false" => Exp::Literal { value: Val::Bool { value: false } },
        "true" => Exp::Literal { value: Val::Bool { value: true } },
        "nil" => Exp::Literal { value: Val::Nil },
        "(" => {
            let e = expr(ts);
            let close_paren = ts.peek().unwrap().to_string();
            if close_paren != ")".to_string() {
                panic!("error: missing right parenthesis");
            }
            e
        },
        _ => {
            if num_re.is_match(&next) {
                Exp::Literal { value: Val::Num { value: i64::from_str(&next).unwrap() } }
            } else if sym_re.is_match(&next) {
                Exp::Literal { value: Val::Sym { value: next }}
            } else {
                panic!("syntax error");
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
        Exp::Command { cmd, args } => Val::Nil {}, /* replace with command exec */
        Exp::Empty {} => Val::Nil {},
    }*/
    Val::Nil {}
}
*/
/*************************************************************************************************/

fn main() {
    /* initialization
            + initialize environment and load environment variables
            + set default PATH
            + load .rushrc
            + set current directory to current user's home
    */
    let mut env: Env = HashMap::new();
    let mut code = String::new();

    loop {
        print!("{} ", PROMPT);
        match io::stdin().read_line(&mut code) {
            Ok(_n) => {
                code.pop(); // remove newline
                let exps = parse(&code);
                //eval_exps(exps, &mut env);
            }
            Err(error) => println!("error: {}", error),
        }
    }
}
