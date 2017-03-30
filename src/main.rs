use std::{env, io};
use std::collections::HashMap;
// mod trie;

/*************************************** definitions *********************************************/

enum Val {
    Num { value: i64 },
    Sym { value: String },
    Bool { value: bool },
    Nil {},
}

impl Clone for Val {
    fn clone(&self) -> Val {
        match *self {
            Val::Num { ref value } => Val::Num { value: value.clone()},
            Val::Sym { ref value } => Val::Sym { value: value.clone()},
            Val::Bool { ref value } => Val::Bool { value: value.clone()},
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
    //Command { cmd: String, args: String}, // variable lookup --> command
    Empty {},
}

/******************************************* parser **********************************************/

fn parse_code(code: &str) -> Vec<Exp> {
    let raw_exps: Vec<&str> = code.split(';' /* | && */).collect();
    let mut exps = Vec::new();
    for e in raw_exps {
        let words = (e as &str).split_whitespace().collect();// this needs to be smarter
        exps.push(parse(&words));
    }
    exps
}

fn parse(words: &Vec<&str>) -> Exp {
    if words.len() > 0 {
        match words[0] {
            "let" => {
                if words.len() != 3 {
                    println!("syntax error: \"let\" expects a name followed by an expression");
                    return Exp::Empty {}; // --> obviously change this
                } else {
                    return Exp::Let {
                        name: words[1].to_string(),
                        e: Box::new(parse(&words[2..].to_vec()))
                    };
                }
            },
            _ => return Exp::Empty {},
        }
    } else {
        Exp::Empty {}
    }
}

/****************************************** evaluator ********************************************/

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
        }
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
