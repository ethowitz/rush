use std::{env, io};
use std::collections::HashMap;
// mod trie;

enum Val {
    Num { value: i64 },
    Sym { value: String },
    Bool { value: bool },
}

type Env = HashMap<String, Val>;

enum Exp {
    Literal { value: Val },
    VarName { name: String },
    If { e1: Box<Exp>, e2: Box<Exp>, e3: Box<Exp> },
    Let { name: String, e: Box<Exp> },
    Command { cmd: String, args: String}, // variable lookup --> command
    Empty {},
}

fn parse_code(code: &str) -> Vec<Exp> {
    let raw_exps: Vec<&str> = code.split(';' /* | && */).collect();
    let mut exps = Vec::new();
    for e in raw_exps {
        let words = (e as &str).split_whitespace().collect();// this needs to be smarter --> write function
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
                    return Exp::Literal {value: "error".to_string()}; // --> obviously change this
                } else {
                    return Exp::Let {
                        name: words[1].to_string(),
                        e: Box::new(parse(&words[2..].to_vec()))
                    };
                }
            },
            _ => return Exp::Literal {value: "lol".to_string()},
        }
    } else {
        Exp::Empty {}
    }
}

pub fn eval_exps(exps: Vec<Exp>, &mut env: Env) {
    for e in exps {
        eval(e, &mut env);
    }
}

fn eval(e: Exp, &mut env: Env) -> Val {
    match e {
        Exp::Literal { name } => name,
        Exp::VarName { name } => {
            match env.get(name) {
                Some(value) => value,
                None => {
                    println!("error: variable \"{}\" not bound", name);
                    return;
                }
            }
        },
        Exp::Let { name, exp } => {
            env.insert(name, eval(exp, &mut env));
            Val::Bool { value: true }
        }
    }
}

fn main() {
    /* initialization
            + initialize environment and load environment variables
            + set default PATH
            + load .rushrc
            + set current directory to current user's home
    */
    let env: Env = HashMap::new();
    let mut code = String::new();


    loop {
        match io::stdin().read_line(&mut code) {
            Ok(_n) => {
                code.pop(); // remove newline
                let exps = parse_code(&code);
                //type::type(&exps);
                eval_exps(&exps, &env);

            }
            Err(error) => println!("error: {}", error),
        }
    }
}
