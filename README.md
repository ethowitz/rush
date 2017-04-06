# Rush

## Description
Rush (Rust Shell) is a command shell and expression-based scripting language
that aims to have intuitive and transparent syntax, making directory navigation
and system administration absolutely blissful.

## Grammar and Syntax
Rush's grammar is LL(1) which gives parsing via a recursive descent parser a
time complexity of O(n). Expressions are always enclosed in square brackets and
can be executed imperatively when separated by semicolons. Once the core of Rush
is completed, I will add example programs/tutorials that demonstrate its
syntax.

## Parser
The parser is implemented as a recursive descent parser, where every
non-terminal component of the grammar has its own function (these functions are
mutually recursive).

## Abstract Syntax and Evaluation
Expressions are represented in the code as enums with the different syntactic
forms as the fields. Values are represented in the code as enums with possible
types NUM (a 64-bit, signed integer), SYM (a string of characters), BOOL, and
NIL.

## Road Map
These are the features I intend to implement in the near-future:
+ Command execution
+ I/O redirection and piping
+ Foreground/background processes
+ Command history and scrolling
+ Cursor movement via the arrow keys
+ SYM
+ for and while loops
+ print
+ Set custom prompt
+ Variable bindings (disallowing the binding of Rush keywords)
+ Config file, environment variables, PATH, and command aliasing
+ Current directory
+ Expressions embedded in commands (e.g. an expression can be evaluated to an
  argument or even the name of a command to be executed)
+ Lambda
+ Predefined functions and lists
+ Invocation of the bash interpreter for backwards compatibility
+ Tab completion via a Trie
+ Status (0 or 1) of last program run
+ Secure password entry
+ Clear, reset, exit, poweroff, sleep
