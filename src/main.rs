use core::panic;

use inkwell::values::BasicMetadataValueEnum::VectorValue;
use std::any::Any;
use std::collections::HashMap;
use std::env::VarError;
use std::env::consts::ARCH;
use std::num::NonZero;
use std::os::unix::process::parent_id;
use std::vec;

#[derive(Debug, Clone)]
struct Param {
    name: String,
    ty: Type,
}

#[derive(Debug, PartialEq, Clone)]
enum Type {
    Int,
    Str,
}
#[derive(Debug, PartialEq, Clone)]
enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}
#[derive(Debug, Clone)]
pub struct Func {
    args: Vec<Param>,
    code: Vec<Stmt>,
    name: String,
}
#[derive(Debug, Clone)]
pub struct Func_call {
    name: String,
    args: Vec<Expr>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Num(i64),
    Id(String),
    Float(f64),
    Str(String),
    Comma,

    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Int(String, Expr),
    Float(String, Expr),
    Str(String, Expr),
    Var(String),
    ReturnStmt(Expr),
    Func(Func),
    Func_call(Func_call),
    Binary(Expr),
    Expr(Expr),
}
#[derive(Debug, PartialEq, Clone)]
enum Token {
    Call,
    Func,
    Return,
    Rparen,
    Lparen,
    Lcurly,
    Rcurly,
    Comma,
    Assign,
    Plus,
    Mines,
    Add,
    Sub,
    Mul,
    Div,
    Semicolon,

    Identifier(String),
    Number(i64),
    String(String),
    Floatval(f64),
    Int,
    Str,
    Float,

    EOF,
}
fn tokenize(code: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();

    let chars: Vec<char> = code.chars().collect();

    let mut i = 0;

    while i < chars.len() {
        let ch: char = chars[i];
        if ch.is_whitespace() {
            i += 1;
            continue;
        }

        match ch {
            '=' => {
                tokens.push(Token::Assign);
                i += 1;
                continue;
            }

            '+' => {
                tokens.push(Token::Add);
                i += 1;
                continue;
            }
            '-' => {
                tokens.push(Token::Mines);
                i += 1;
                continue;
            }
            '*' => {
                tokens.push(Token::Mines);
                i += 1;
                continue;
            }
            '/' => {
                tokens.push(Token::Mines);
                i += 1;
                continue;
            }
            ')' => {
                tokens.push(Token::Rparen);
                i += 1;
                continue;
            }
            '}' => {
                tokens.push(Token::Rcurly);
                i += 1;
                continue;
            }
            ',' => {
                tokens.push(Token::Comma);
                i += 1;
                continue;
            }
            '{' => {
                tokens.push(Token::Lcurly);
                i += 1;
                continue;
            }
            '(' => {
                tokens.push(Token::Lparen);
                i += 1;
                continue;
            }
            _ => {}
        }

        if ch.is_ascii_digit() {
            let mut num_str = String::new();

            while i < chars.len() && chars[i].is_ascii_digit() {
                num_str.push(chars[i]);
                i += 1
            }
            let num = num_str.parse::<i64>().unwrap();
            tokens.push(Token::Number(num));
            continue;
        }

        if ch == '"' {
            i += 1;
            let mut str = String::new();
            while chars[i] != '"' && i < chars.len() {
                str.push(chars[i]);
                i += 1;
            }

            tokens.push(Token::String(str));
            i += 1;

            continue;
        }

        if ch.is_alphabetic() || ch == '_' && ch != '"' {
            let mut ident_str = String::new();
            while i < chars.len() && (chars[i].is_alphabetic() || chars[i] == '_') {
                ident_str.push(chars[i]);
                i += 1;
            }

            match ident_str.as_str() {
                "return" => tokens.push(Token::Return),
                "int" => tokens.push(Token::Int),
                "str" => tokens.push(Token::Str),
                "func" => tokens.push(Token::Func),
                "call" => tokens.push(Token::Call),
                _ => tokens.push(Token::Identifier(ident_str)),
            }
            continue;
        }
    }
    tokens.push(Token::EOF);
    tokens
}
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    func_names: Vec<Func>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            pos: 0,
            func_names: Vec::new(),
        }
    }

    fn current(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) -> &Token {
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        &self.tokens[self.pos - 1]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.pos - 1]
    }
    fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while *self.current() != Token::EOF {
            statements.push(self.parse_statement());
        }

        statements
    }

    fn parse_return(&mut self) -> Stmt {
        self.advance();
        let expr = self.parse_primary();

        Stmt::ReturnStmt(expr)
    }
    fn parse_int(&mut self) -> Stmt {
        self.advance();

        let name = match self.advance().clone() {
            Token::Identifier(name) => name,
            _ => panic!("Expected identifier"),
        };

        if *self.advance() != Token::Assign {
            panic!("Expected '='");
        }

        let expr = self.parse_expr();

        Stmt::Int(name, expr)
    }
    fn parse_func(&mut self) -> Stmt {
        self.advance();

        let mut args: Vec<Param> = Vec::new();
        let mut code_token: Vec<Token> = Vec::new();
        let name = match self.advance().clone() {
            Token::Identifier(name) => name,
            _ => panic!("Expected identifier"),
        };

        if *self.advance() != Token::Lparen {
            panic!("Expected '('")
        }

        while *self.current() != Token::Rparen {
            if let typee = self.parse_func_args() {
                let name = match self.advance().clone() {
                    Token::Identifier(name) => name,
                    _ => panic!("dms.d;/,sdml"),
                };
                args.push(Param { name, ty: typee });
            };

            if *self.current() == Token::Comma {
                self.advance();
            }
        }

        self.advance();
        if *self.advance() != Token::Lcurly {
            panic!("Expected ")
        }
        while *self.current() != Token::Rcurly {
            code_token.push(self.current().clone());
            self.advance();
        }

        self.advance();
        code_token.push(Token::EOF);

        let mut parser = Parser::new(code_token.clone());

        println!("{:?}", code_token.clone());
        Stmt::Func(Func {
            args,
            code: parser.parse(),
            name,
        })
    }

    fn parse_func_args(&mut self) -> Type {
        match self.advance().clone() {
            Token::Int => Type::Int,
            Token::Str => Type::Str,

            _ => panic!("s"),
        }
    }
    fn parse_str(&mut self) -> Stmt {
        self.advance();

        let name = match self.advance().clone() {
            Token::Identifier(name) => name,
            _ => panic!("Expected identifier"),
        };

        if *self.advance() != Token::Assign {
            panic!("Expected '='");
        }

        let expr = self.parse_primary();

        if let Expr::Str(_) = &expr {
            Stmt::Str(name, expr)
        } else {
            panic!("Expected string");
        }
    }
    fn parse_statement(&mut self) -> Stmt {
        match self.current().clone() {
            Token::Int => self.parse_int(),
            Token::Str => self.parse_str(),
            Token::Return => self.parse_return(),
            Token::Func => self.parse_func(),

            _ => {
                let expr = self.parse_expr();
                Stmt::Expr(expr)
            }
        }
    }
    fn parse_exp3r(&mut self) -> Expr {
        let left = match self.advance().clone() {
            Token::Number(val) => Expr::Num(val),
            Token::Identifier(name) => Expr::Id(name),
            Token::String(str) => Expr::Str(str),
            _ => panic!("error{:?}", self.current()),
        };

        left
    }
    fn parse_primary(&mut self) -> Expr {
        let token = self.advance().clone();
        match token {
            Token::Number(val) => Expr::Num(val),
            Token::Identifier(name) => Expr::Id(name),
            Token::String(str) => Expr::Str(str),
            _ => panic!("error{:?}", token),
        }
    }

    fn parse_expr(&mut self) -> Expr {
        let mut left = self.parse_term();
        while matches!(self.current(), Token::Mines | Token::Add) {
            let op = match self.advance() {
                Token::Add => BinaryOp::Add,
                Token::Mines => BinaryOp::Sub,
                _ => unreachable!(),
            };

            let right = self.parse_term();

            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            }
        }
        left
    }
    fn parse_term(&mut self) -> Expr {
        let mut left = self.parse_primary();

        while matches!(self.current(), Token::Mul | Token::Div) {
            let op = match self.advance() {
                Token::Mul => BinaryOp::Mul,
                Token::Div => BinaryOp::Div,
                _ => unreachable!(),
            };

            let right = self.parse_primary();

            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        left
    }
    fn back(&mut self) {
        if self.pos > 0 {
            self.pos -= 1;
        }
    }
}
pub fn ready_code(code: &str) -> Vec<Stmt> {
    let tokens = tokenize(code);

    let mut parser = Parser::new(tokens);
    let ast = parser.parse();

    ast
}
fn main() {
    let my_code = r#"func hello(int i,str he){int i = 23}"#;
    let tokens = tokenize(my_code);
    println!("{:?}", tokens.clone());

    let mut parser = Parser::new(tokens.clone());
    let emc = parser.parse();
    println!("{:?}", emc);
    println!("{:?}", parser.parse());
}


