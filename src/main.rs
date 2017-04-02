extern crate regex;

use regex::Regex;
use std::{env, io};
use std::collections::HashMap;


// mod trie;

/********************************* definitions ***************************************************/

enum Val {
    Num { value: i64 },
    Sym { value: String },
    Bool { value: bool },
    Nil {},
}

impl Clone for Val {
    fn clone(&self) -> Val {
        match *self {
            Val::Num { ref value } => Val::Num { value: value.clone() },
            Val::Sym { ref value } => Val::Sym { value: value.clone() },
            Val::Bool { ref value } => Val::Bool { value: value.clone() },
            Val::Nil {} => Val::Nil{},
        }
    }
}

type Env = HashMap<String, Val>;
const PROMPT: &'static str = ">>";

enum Exp {
    Literal { value: Val },
    VarName { name: String },
    //If { e1: Box<Exp>, e2: Box<Exp>, e3: Box<Exp> },
    Let { name: String, e: Box<Exp> },
    Command { cmd: String, args: Vec<String> }, // catch-all
    Empty {},
}

/************************************* parser ****************************************************/

fn parse_code(code: &str) -> Vec<Exp> {
    let concrete_exps: Vec<&str> = code.split(';' /* | && */).collect();

    let mut exps = Vec::new();
    for e in concrete_exps {
        let mut tokens = tokenize(e);
        exps.push(parse(tokens));
    }
    exps
}

fn tokenize(exp: &str) -> Vec<&str> {
    let re = Regex::new(r" +|\n+").unwrap();
    re.split(exp).collect()
}

fn parse(exp: Vec<&str>) -> Exp {
    /* regex definitions */
    let var_name_re = Regex::new(r":^[a-z_]\\w*$").unwrap();
    let sym_re = Regex::new("\"^[\\w\\-. ]+$\"").unwrap();

    Exp::Empty {}
}

fn expression(tokens: &mut Vec<&str>) {

}

fn uni_op(tokens: &mut Vec<&str>) {

}

fn bin_op(tokens: &mut Vec<&str>) {

}

fn def(tokens: &mut Vec<&str>) {

}

/*********************************** evaluator ***************************************************/

fn eval_exps(exps: Vec<Exp>, env: &mut Env) {
    for e in exps {
        eval(e, env);
    }
}

fn eval(e: Exp, env: &mut Env) -> Val {
    match e {
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
    }
}

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
                let exps = parse_code(&code);
                //type::type(&exps);
                eval_exps(exps, &mut env);
            }
            Err(error) => println!("error: {}", error),
        }
    }
}
