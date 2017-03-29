use std::{env, io, path::Path};

/*

Lifecycle: check syntax and convert to AST --> check types of AS --> run!

+ Hash map for variable environment
  --> Environment variables are variables that are persistent across instances of the shell
  --> aliasing
+ Built-in syntax: get/set variables, print environment, exit, arithmetic operations, for,
  while, if, echo (print? show?), sleep/poweroff, functions/lambda, ;, &&, fork (instead of &),
  fg/bg (maybe lil taskbar sorta thing at the top with fg highlighted OR list when prompted to)
  --> this syntax does not have to be equivalent to above, but functionality should be more or less
      the same as in bash
  --> be smart about what words are reserved (i.e. don't reserve words that might be used as
      program names)
  --> split into possible syntactic forms separated by && or ; or nl with binary as its own SF
      (should act as the catch-all)
  --> things in parens should be treated as its own expression, with expressions evaluated from
      inside out --> expressions separated by ;/&&/nl are evaluated sequentially
  --> will need to parse to convert into abstract syntax tree
+ Type system
  --> definitely not inferred
  --> NUM, BOOL, SYM (it is legal to attempt to invoke a SYM as a command), list
+ Fast tab completion with trie --> keep all legal words in trie and continue to update
+ Stream redirection
+ status (0 or 1) of last program run
+ command history
+ secure password entry
+ script security
  --> permissions on scripts somehow? how much of this is managed by OS?
  --> prevent fork bombs
+ fun easter eggs
  --> progress ASCII art
  --> C O L O R S
  --> COLORIZED matrix + maybe some other ASCII art (maybe for screensaver type feature?)
  --> exploit the wonders of unicode to make cool art/effects
*/

fn main() {
    /* initialization
            + initialize environment
            + set default PATH
            + set current directory to current user's home
    */
    let mut command = String::new();

    loop {
        match io::stdin().read_line(&mut command) {
            Ok(n) => {
                command.pop();
                process_command(&command);
            }
            Err(error) => println!("error: {}", error),
        }
    }
}



fn process_command(command: &str) {
    match command {
        "cd" => println!("NOICE"),
        _ => println!("error: command not found"),
    }
}
