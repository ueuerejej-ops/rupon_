use core::panic;
use std::any::Any;
use std::collections::HashMap;
use std::env::VarError;
use std::env::consts::ARCH;
use std::num::NonZero;
use std::os::unix::process::parent_id;
use std::vec;

use inkwell::values::BasicMetadataValueEnum::VectorValue;

#[derive(Debug, Clone)]
enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}
#[derive(Debug, Clone)]
pub struct Func {
    args: Vec<Expr>,
    code: Vec<Stmt>,
    name: String,
}
#[derive(Debug, Clone)]
pub struct Func_call {
    name: String,
    args: Vec<Expr>,
}
#[derive(Debug, Clone)]
pub enum Expr {
    Num(i64),
    Id(String),
    Float(f64),
    str(String),
    Plus,
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
    Plus(Expr),
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
                tokens.push(Token::Plus);
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

    // fn get_func(&self,name: &str)->Func{
    //     for func in self.func_names{
    //         if func.name == name{

    //         }
    //     }
    // }
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
    fn parse_statement(&mut self) -> Stmt {
        let token = self.current().clone();
        match token {
            Token::Int => {
                self.advance();

                if let Token::Identifier(name) = self.advance().clone() {
                    if *self.advance() == Token::Assign {
                        let expr = self.parse_expr();
                        Stmt::Int(name, expr)
                    } else {
                        panic!("Ошибка парсинга: Ожидался знак '='");
                    }
                } else {
                    panic!("Ошибка парсинга: Ожидалось имя переменной");
                }
            }

            Token::Return => {
                self.advance();
                let expr = self.parse_expr();

                self.tokens.push(Token::EOF);
                Stmt::ReturnStmt(expr)
            }
            Token::Number(num) => {
                let expr = self.parse_expr();

                Stmt::Binary(expr)
            }

            Token::Func => {
                self.advance();

                let mut token_args_vec: Vec<Expr> = Vec::new();
                if let Token::Identifier(name) = self.advance().clone() {
                    let mut args: Vec<Expr> = Vec::new();
                    if *self.advance() == Token::Lparen {
                        while *self.current() != Token::Rparen {
                            token_args_vec.push(self.parse_expr_for_binary());

                            if *self.current() == Token::Comma {
                                self.advance();
                            }
                        }

                        //    if token_args_vec.len() == 0{

                        // //    }
                        // if !token_args_vec.is_empty() && token_args_vec[0] == Token::Comma{
                        //     panic!("sdsdsd")
                        // }
                        // for (i,token) in token_args_vec.iter().enumerate(){

                        //     if *token != Token::Comma{

                        //         args.push(self.parse_expr_token(token.clone()));
                        //     }else{

                        //         args.push(Expr::Comma);
                        //     }
                        // }
                    } else {
                        panic!("sdf")
                    }

                    self.advance();
                    if *self.current() == Token::Lcurly {
                        self.advance();
                        let mut vec_token: Vec<Token> = Vec::new();

                        let mut code: Vec<Stmt> = Vec::new();
                        while *self.current() != Token::Rcurly {
                            vec_token.push(self.current().clone());
                            self.advance();
                        }
                        self.advance();
                        println!("{:?}", vec_token);
                        vec_token.push(Token::EOF);
                        let mut parser: Parser = Parser::new(vec_token);
                        Stmt::Func(Func {
                            args: token_args_vec,
                            code: parser.parse(),
                            name,
                        })
                    } else {
                        panic!("sas")
                    }
                } else {
                    panic!("assdzd")
                }
            }

            Token::Plus => {
                let expr = self.parse_expr_for_binary();

                Stmt::Plus(expr)
            }

            Token::Str => {
                self.advance();
                if let Token::Identifier(name) = self.advance().clone() {
                    if *self.advance() == Token::Assign {
                        let expr = self.parse_expr();
                        Stmt::Str(name, expr)
                    } else {
                        panic!("SD")
                    }
                } else {
                    panic!("sd")
                }
            }

            Token::Identifier(name) => {
                if *self.advance() != Token::Lparen {
                    return Stmt::Var(name);
                }
                self.advance();

                self.advance();
                let mut args: Vec<Expr> = Vec::new();
                while *self.current() != Token::Rparen {
                    args.push(self.parse_expr());
                    if *self.current() == Token::Comma {
                        self.advance();
                    }
                }
                self.advance();

                Stmt::Func_call(Func_call { name: name, args })
            }
            _ => panic!(
                "Ошибка парсинга: Неизвестная инструкция {:?}",
                self.current()
            ),
        }
    }
    fn parse_expr(&mut self) -> Expr {
        let left = match self.advance().clone() {
            Token::Number(val) => Expr::Num(val),
            Token::Identifier(name) => Expr::Id(name),
            Token::String(str) => Expr::str(str),
            _ => panic!("error{:?}", self.current()),
        };

        left
    }
    fn back(&mut self) {
        if self.pos > 0 {
            self.pos -= 1;
        }
    }
    fn BinaryOp(&mut self) -> BinaryOp {
        match *self.current() {
            Token::Plus => BinaryOp::Add,
            Token::Div => BinaryOp::Div,
            Token::Mul => BinaryOp::Mul,
            Token::Sub => BinaryOp::Sub,
            _ => panic!("error"),
        }
    }
    fn parse_expr_for_binary(&mut self) -> Expr {
        self.back();
        let left = self.parse_expr();

        if *self.current() == Token::Plus {
            self.advance();
            let right = self.parse_expr();

            return Expr::Binary {
                left: Box::new(left),
                op: BinaryOp::Add,
                right: Box::new(right),
            };
        }

        left
    }
}
pub fn ready_code(code: &str) -> Vec<Stmt> {
    let tokens = tokenize(code);

    let mut parser = Parser::new(tokens);
    let ast = parser.parse();

    ast
}
fn main() {
    let my_code = r#"func yhello(){ i + 12 + hello +233223}"#;
    let tokens = tokenize(my_code);
    println!("{}", my_code);

    let code2 = r#"func hewllo(){i}"#;
    let tokens2 = tokenize(code2);
    println!("{:?}", tokens2);
    println!("{:?}", tokens);
    let mut parser = Parser::new(tokens.clone());
    let mut parser2 = Parser::new(tokens2.clone());
    let emc = parser.parse();
    let emc2 = parser2.parse();
    println!("{:?}", emc);
}

