use core::panic;

mod token;
use token::Token;
use token::tokenize;

#[derive(Debug, Clone)]
struct Param<'a>{
    name: &'a str,
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
pub struct Func<'a> {
    args: Vec<Param<'a>>,
    code: Vec<Stmt<'a>>,
    name: &'a str,
}
#[derive(Debug, Clone)]
pub struct Func_call<'a> {
    name: &'a str,
    args: Vec<Expr<'a>>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Expr<'a>{
    Num(i64),
    Id(&'a str),
    Float(f64),
    Str(&'a str),
    Comma,

    Binary {
        left: Box<Expr<'a>>,
        op: BinaryOp,
        right: Box<Expr<'a>>,
    },
}
#[derive(Debug,Clone)]

pub struct Var<'a>{
    tipe: Type,
    value: Expr<'a>,
    name: &'a str,
}
#[derive(Debug, Clone)]
pub enum Stmt<'a>{
    Int(Var<'a>),
    Float(& 'a str, Expr<'a>),
    Str(&'a str,Expr<'a>),
    Var(&'a str),
    ReturnStmt(Expr<'a>),
    Func(Func<'a>),
    Func_call(Func_call<'a>),
    Binary(Expr<'a>),
    Expr(Expr<'a>),
}
struct Parser<'a>{
    tokens: Vec<Token<'a>>,
    pos: usize,
    func_names: Vec<Func<'a>>,
}

impl <'a>Parser <'a>{
    fn new(tokens: Vec<Token<'a>>) -> Self {
        Parser {
            tokens,
            pos: 0,
            func_names: Vec::new(),
        }
    }

    fn current(&self) -> Token<'a> {
        self.tokens[self.pos]
    }

   fn advance(&mut self) -> Token<'a> {
    let tok = self.tokens[self.pos].clone();
    self.pos += 1;
    tok
}

    fn previous(&self) -> Token<'a> {
        self.tokens[self.pos - 1]
    }
    fn parse(&  mut self) -> Vec<Stmt<'a>> {
        let mut statements = Vec::new();
        while self.current() != Token::EOF {
            statements.push(self.parse_statement());
        }

        statements
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

        let expr = self.chech_expr(Type::Int);

        Stmt::Int(Var  {
            tipe: Type::Int,
            value: expr,
            name,
        })
    }
    fn chech_expr(&mut self, tipe: Type) -> Expr<'a> {
        let expr = self.parse_expr();
        match (tipe.clone(), expr.clone()) {
            (Type::Int, Expr::Num(_))
            | (Type::Int, Expr::Id(_))
            | (Type::Int, Expr::Binary { .. })
            | (Type::Str, Expr::Str(_))
            | (Type::Str, Expr::Id(_)) => {expr},

           _=> panic!("Cannot put {:?} to {:?}", tipe, expr)

        }
     
    }
    fn parse_func_call(&mut self) -> Stmt<'a> {
        let mut args: Vec<Expr> = Vec::new();

        let name = match self.advance().clone() {
            Token::Identifier(name) => name,
            _ => panic!("Expected identifier"),
        };

        self.advance();
        while self.current() != Token::Rparen {
            if self.current() == Token::Comma {
                self.advance();
            }
            args.push(self.parse_expr());
        }
        self.advance();
        Stmt::Func_call(Func_call { name, args })
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

        let mut parser = Parser::new(code_token.clone());

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

        let expr = self.parse_primary();

        if let Expr::Str(_) = &expr {
            Stmt::Str(name, expr)
        } else {
            panic!("Expected string");
        }
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
                Stmt::Expr(expr)
            }
        }
    }

    fn parse_primary(&mut self) -> Expr<'a> {
        let token = self.advance();
        match token {
            Token::Number(val) => Expr::Num(val),
            Token::Identifier(name) => Expr::Id(name),
            Token::String(str) => Expr::Str(str),
            _ => panic!("error{:?}", token),
        }
    }

        fn parse_primary_for_bianry(&mut self) -> Expr<'a> {
        let token = self.advance();
        match token {
            Token::Number(val) => Expr::Num(val),
            Token::Identifier(name) => Expr::Id(name),
       
                        Token::Lparen=>{
                          let expr =  self.parse_expr();
                            if self.current() != Token::Rparen{
                                panic!("Expected ')'")
                            }
                            self.advance();
                            expr


                        },
            _ => panic!("error{:?}", token),
        }
    }
    fn parse_expr(&mut self) -> Expr<'a> {
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
    fn parse_term(&mut self) -> Expr<'a>{
        let mut left = self.parse_primary_for_bianry();

        while matches!(self.current(), Token::Mul | Token::Div) {
            let op = match self.advance() {
                Token::Mul => BinaryOp::Mul,
                Token::Div => BinaryOp::Div,
                _ => unreachable!(),
            };

            let right = self.parse_primary_for_bianry();

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
pub fn ready_code<'a>(code: &'a str) -> Vec<Stmt<'a>> {
    let tokens = tokenize(code);

    let mut parser = Parser::new(tokens);
    let ast = parser.parse();

    ast
}
fn main() {
    let my_code = r#"1 +1 *2 +2"#;
    let tokens = tokenize(my_code);
    println!("{:?}", tokens.clone());

    let mut parser = Parser::new(tokens.clone());
    let emc = parser.parse();
    println!("{:?}", emc);
    println!("{:?}", parser.parse());
}


