use core::panic;

use crate::arena::expr_add;
use crate::token::tokenize;
use crate::token::Token;
use crate::arena::Arena;
#[derive(Debug, Clone)]
struct Param<'a>{
    name: &'a str,
    ty: Type,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
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
pub struct Func<'a> {
    args: Vec<Param<'a>>,
    code: Vec<Stmt<'a>>,
    name: &'a str,
}
#[derive(Debug, Clone)]
pub struct Func_call<'a> {
    name: &'a str,
    args: Vec<*mut Expr<'a>>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Expr<'a>{
    Num(i64),
    Id(&'a str),
    Float(f64),
    Str(&'a str),
    Binary(*mut Expr<'a>,BinaryOp,*mut Expr<'a>
    
    ),
    Comma,

}

#[derive(Debug, PartialEq, Clone)]
struct Binary<'a>{

        left: *mut  Expr<'a>,
        op: BinaryOp,
        right: *mut Expr<'a>
}
#[derive(Debug,Clone)]

pub struct Var<'a>{
    pub tipe: Type,
    pub value: *mut Expr<'a>,
    pub name: &'a str,
}
#[derive(Debug, Clone)]
pub enum Stmt<'a>{
    Int(Var<'a>),
    Float(& 'a str, &'a Expr<'a>),
    Str(&'a str,*mut Expr<'a>),
    Var(&'a str),
    ReturnStmt(*mut Expr<'a>),
    Func(Func<'a>),
    Func_call(Func_call<'a>),
    Binary(&'a Expr<'a>),
    Expr(*mut Expr<'a>),
}
#[derive(Debug,Clone)]
struct Parser<'a>{
    arena: *mut Arena<'a>,
    tokens: Vec<Token<'a>>,
    pos: usize,
    func_names: Vec<Func<'a>>,
}



impl <'a>Parser <'a>{
    fn new(arena: * mut Arena<'a>,tokens: Vec<Token<'a>>) -> Self {
Parser { arena: arena ,tokens, pos: 0, func_names: Vec::new() }
    }

    fn current(& self) -> Token<'a> {
        self.tokens[self.pos]
    }

   fn advance(&mut self) -> Token<'a> {
    let tok = self.tokens[self.pos].clone();
    self.pos += 1;
    tok
}

    fn previous(& self) -> Token<'a> {
        self.tokens[self.pos - 1]
    }

    fn parse(&  mut self) -> Vec<Stmt<'a>> {
        let mut statements = Vec::new();
        while self.current() != Token::EOF {
            statements.push(self.parse_statement());
        }

        statements
    }
      fn next(&mut self) -> Token<'a> {
        self.tokens[self.pos + 1]
    }
  fn parse_statement(&  mut self) -> Stmt<'a>{
        match self.current() {
            Token::Int => self.parse_int(),
            Token::Str => self.parse_str(),
            Token::Return => self.parse_return(),
            Token::Func => self.parse_func(),
            Token::Identifier(name) => {
                if self.next() == Token::Lparen {
                    self.parse_func_call()
                } else {
                    panic!("sas")
                }
            }

            _ => {
                let expr = self.parse_expr();

unsafe {
    if let Expr::Str(_) = &*expr {
        Stmt::Expr( expr)
    } else {
        panic!("Expected string");
    }
}
            }
        }
    }
    fn parse_return(&mut self) -> Stmt<'a> {
        self.advance();
        let expr = self.parse_primary();

        Stmt::ReturnStmt( expr)
    }
    fn parse_int(& mut self) -> Stmt<'a> {
        self.advance();

        let name = match self.advance().clone() {
            Token::Identifier(name) => name,
            _ => panic!("Expected identifier"),
        };

        if self.advance() != Token::Assign {
            panic!("Expected '='");
        }

        let expr = self.check_expr(Type::Int);

        Stmt::Int(Var  {
            tipe: Type::Int,
            value: expr,
            name,
        })
    }
    fn check_expr(&mut self, tipe: Type) -> *mut Expr<'a> {
    let expr = self.parse_expr();

    unsafe {
        match (tipe.clone(), &*expr) {
            (Type::Int, Expr::Num(_))
            | (Type::Int, Expr::Id(_))
            | (Type::Int, Expr::Binary { .. })
            | (Type::Str, Expr::Str(_))
            | (Type::Str, Expr::Id(_)) => expr,

            _ => panic!("Cannot put {:?} to {:?}", tipe.clone(), &*expr),
        }
    }
}
    fn parse_func_call(&mut self) -> Stmt<'a> {
        let mut argument = Vec::new();

        let name = match self.advance().clone() {
            Token::Identifier(name) => name,
            _ => panic!("Expected identifier"),
        };

        self.advance();
        while self.current() != Token::Rparen {
            if self.current() == Token::Comma {
                self.advance();
            }
            argument.push(self.parse_expr());
        }
        self.advance();
        Stmt::Func_call(Func_call { name, args:argument })
    }
    fn parse_func(&mut self) -> Stmt<'a> {
        self.advance();

        let mut args: Vec<Param> = Vec::new();
        let mut code_token: Vec<Token<'a>> = Vec::new();
        let name = match self.advance() {
            Token::Identifier(name) => name,
            _ => panic!("Expected identifier"),
        };

        if self.advance() != Token::Lparen {
            panic!("Expected '('")
        }

        while self.current() != Token::Rparen {
            if let typee = self.parse_func_args() {
                let name = match self.advance() {
                    Token::Identifier(name) => name,
                    _ => panic!("Expected identifier"),
                };
                args.push(Param { name, ty: typee });
            };

            if self.current() == Token::Comma {
                self.advance();
            }
        }

        self.advance();
        if self.advance() != Token::Lcurly {
            panic!("Expected Lcurly ")
        }
        while self.current() != Token::Rcurly {
            code_token.push(self.current());
            self.advance();
        }

        self.advance();
        code_token.push(Token::EOF);

        let mut parser = Parser::new(self.arena,code_token.clone());

        println!("{:?}", code_token);
        Stmt::Func(Func {
            args,
            code: parser.parse(),
            name: name,
        })
    }

    fn parse_func_args(&mut self) -> Type {
        match self.advance().clone() {
            Token::Int => Type::Int,
            Token::Str => Type::Str,

            _ => panic!("Expected type "),
        }
    }
    fn parse_str(&mut self) -> Stmt<'a> {
        self.advance();

        let name = match self.advance().clone() {
            Token::Identifier(name) => name,
            _ => panic!("Expected identifier"),
        };

        if self.advance() != Token::Assign {
            panic!("Expected '='");
        }

        let mut expr = self.parse_primary();
unsafe {
    if let Expr::Str(_) = &*expr {
        Stmt::Str(name, expr)
    } else {
        panic!("Expected string");
    }
}
    }

 fn parse_primary_for_bianry(&mut self) -> *mut Expr<'a> {
    let token = self.advance();

    match token {
        Token::Number(val) => {
            expr_add(self.arena, Expr::Num(val))
        }

        Token::Identifier(name) => {
            expr_add(self.arena, Expr::Id(name))
        }

        Token::Lparen => {
            let expr = self.parse_expr();

            if self.current() != Token::Rparen {
                panic!("Expected ')'");
            }

            self.advance();

            expr
        }

        _ => panic!("error {:?}", token),
    }
}
  fn parse_expr(&mut self) ->*mut Expr<'a>{
        let mut left = self.parse_term();
        while matches!(self.current(), Token::Mines | Token::Add) {
            let op = match self.advance() {
                Token::Add => BinaryOp::Add,
                Token::Mines => BinaryOp::Sub,
                _ => unreachable!(),
            };

            let right = self.parse_term();

            let expr = Expr::Binary( left, op, right );
            left = expr_add(self.arena, expr);
        }
        left
    }
        
    fn parse_term(&mut self) ->*mut Expr<'a>{
        let mut left = self.parse_primary_for_bianry();

        while matches!(self.current(), Token::Mul | Token::Div) {
            let op = match self.advance() {
                Token::Mul => BinaryOp::Mul,
                Token::Div => BinaryOp::Div,
                _ => unreachable!(),
            };

            let right = self.parse_primary_for_bianry();

       let expr = Expr::Binary(left, op, right);
       left = expr_add(self.arena, expr);
        }
        left
    }
    fn parse_primary(& mut self) -> *mut Expr<'a> {
        let token = self.advance();
        let   expr= match token {
            Token::Number(val) => Expr::Num(val),
            Token::Identifier(name) => Expr::Id(name),
            Token::String(str) => Expr::Str(str),
            _ => panic!("error{:?}", token),
        };
       expr_add(self.arena, expr)
    }

    fn fhdf(&mut  self){
        self.advance();
        self.previous();
        self.parse_primary();
        self.previous();
        self.parse_primary();
    }
}

pub fn ready_code<'a>(arena: *mut Arena<'a>,code: &'a str)-> Vec<Stmt<'a>>{
      let tokens = tokenize(code);
      let mut parser = Parser::new(arena, tokens);
      parser.parse()
}

fn main() {
    let mut arena = Arena::new(2000);

    let code = r#"int i = 10 str hello = "dsd""#;

    let tokens = tokenize(code);

    let mut parser = Parser::new(
        &mut arena as *mut Arena,
        tokens,
    );

    println!("{:?}",parser.parse());
}

