#[warn(unused)]
use core::panic;

use crate::arena::expr_add;
use crate::code_gen::Compiler;
use crate::code_gen::StringInterner;
use crate::code_gen::SymbolHash;
use crate::code_gen::Varaibeldata;
use crate::parser;
use crate::token;
use crate::token::tokenize;
use crate::token::Token;
use crate::arena::Arena;
#[derive(Debug, Clone,PartialEq)]
pub struct Param<'a>{
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
#[derive(Debug, Clone,PartialEq)]
pub struct Func<'src> {
    args: Vec<Param<'src>>,
    code: Vec<Stmt<'src>>,
    name: &'src str,
    ty: Option<Type>,
    returnv: Option<Expr<'src>>,
}
#[derive(Debug, Clone,PartialEq)]
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



pub struct Var<'a>{
    pub tipe: Type,
    pub value: *mut Expr<'a>,
    pub name: &'a str,
}
#[derive(Debug, Clone,PartialEq)]

pub enum Stmt<'a>{
      Assign {
    name: &'a str,
    value: *mut Expr<'a>,
},
          Int(Var<'a>),
    Float(& 'a str, &'a Expr<'a>),
    Str(Var<'a>),
    Var(&'a str),
    ReturnStmt(*mut
         Expr<'a>),
    Func(Func<'a>),
    Func_call(Func_call<'a>),
    Binary(&'a Expr<'a>),
    Expr(*mut Expr<'a>),
}
#[derive(Debug,PartialEq)]
struct Parser<'src,'a>{
    arena: *mut Arena,
    vars_name: &'a mut StringInterner<'src>,

    tokens: Vec<Token<'src>>,
    pos: usize,
    func_names: Vec<Func<'src>>,
}



impl <'src,'a>Parser <'src,'a>{
    fn new(arena: * mut Arena,strint: &'a mut StringInterner<'src>,tokens: Vec<Token<'src>>) -> Self {
Parser { arena: arena ,tokens, pos: 0, func_names: Vec::new() ,vars_name:  strint}
    }

    fn current(& self) -> Token<'src> {
        self.tokens[self.pos]
    }

   fn advance(&mut self) -> Token<'src> {
    let tok = self.tokens[self.pos].clone();
    self.pos += 1;
    tok
}


    fn previous(& self) -> Token<'src> {
        self.tokens[self.pos - 1]
    }

    fn parse(&  mut self) -> Vec<Stmt<'src>> {
        let mut statements = Vec::new();
        while self.current() != Token::EOF {
            statements.push(self.parse_statement());
        }

        statements
    }
      fn next(&mut self) -> Token<'src> {
        self.tokens[self.pos + 1]
    }
  fn parse_statement(&  mut self) -> Stmt<'src>{
        match self.current() {
            Token::Int => self.parse_int(),
            Token::Str => self.parse_str(),
            Token::Return => self.parse_return(),
            // Token::Func => self.parse_func(),
            Token::Identifier(name) => {
                if self.next() == Token::Lparen {
                    self.parse_func_call()
                } else {
                    self.setvar(name)
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
    fn parse_return(&mut self) -> Stmt<'src> {
        self.advance();
        let expr = self.parse_primary();

        Stmt::ReturnStmt( expr)
    }

    fn parse_int(& mut self) -> Stmt<'src> {
        self.advance();

        let mut name = match self.advance().clone() {
            Token::Identifier(name) => name,
            _ => panic!("Expected identifier"),
        };

        if self.advance() != Token::Assign {
            panic!("Expected '='");
        }

        let expr = self.check_expr(Type::Int);

     let id =    self.vars_name.itern(name);

        Stmt::Int(Var  {
            tipe: Type::Int,
            value: expr,
            name,
        })
    }
    fn check_expr(&mut self, tipe: Type) -> *mut Expr<'src> {
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
    fn parse_func_call(&mut self) -> Stmt<'src>{
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
//     fn parse_func(&mut self) -> Stmt<'src> {
//         self.advance();

//         let mut args: Vec<Param> = Vec::new();
//         let mut code_token: Vec<Token<'src>> = Vec::new();
//         let name = match self.advance() {
//             Token::Identifier(name) => name,
//             _ => panic!("Expected identifier"),
//         };

//         if self.advance() != Token::Lparen {
//             panic!("Expected '('")
//         }

//         while self.current() != Token::Rparen {
//             if let typee = self.parse_func_args() {
//                 let name = match self.advance() {
//                     Token::Identifier(name) => name,
//                     _ => panic!("Expected identifier"),
//                 };
//                 args.push(Param { name, ty: typee });
//             };

//             if self.current() == Token::Comma {
//                 self.advance();
//             }
//         }

//         self.advance();
//         if self.advance() != Token::Lcurly {
//             panic!("Expected Lcurly ")
//         }
//         while self.current() != Token::Rcurly {
//             code_token.push(self.current());
//             self.advance();
//         }

//         self.advance();
//         code_token.push(Token::EOF);

//         let mut parser = Parser::new(self.arena,self.vars_name,code_token);


//         let stmts = parser.parse();
//         let mut returnv = None;

//         let mut return_type: Option<Type> 
//          = None;
// for stmt in &stmts {
//     if let Stmt::ReturnStmt(value) = stmt {
//         unsafe {
//             returnv = Some((*(*value)).clone());
//             match returnv {
//                 Some(Expr::Num(_))=>{return_type = Some(Type::Int);}
//                  Some(Expr::Str(_))=>{return_type = Some(Type::Str);}
//                  Some(Expr::Id(_))=>{return_type = Some(Type::Int);}
//             }
//         }
//         break;
//     }
// }

// Stmt::Func(Func {
//     args,
//     code: stmts,
//     name,
//     returnv,
//     ty: None,
// })

//         }
    fn parse_func_args(&mut self) -> Type {
        match self.advance().clone() {
            Token::Int => Type::Int,
            Token::Str => Type::Str,

            _ => panic!("Expected type "),
        }
    }
    fn setvar(&mut self,name: &'src str)->Stmt<'src>{
        if self.vars_name.string.contains(&name){
            self.advance();
            if self.advance() != Token::Assign{
                panic!("Expected =")
            }
            let value = self.parse_primary();
            Stmt::Assign { name, value: value }
        }else {
            panic!("cannot get name")
        }

    }
       
    fn parse_str(&mut self) -> Stmt<'src> {
        self.advance();

        let name = match self.advance().clone() {
            Token::Identifier(name) => name,
            _ => panic!("Expected identifier"),
        };

        if self.advance() != Token::Assign {
            panic!("Expected '='");
        }

        let expr = self.parse_primary();
unsafe {
    self.vars_name.string.push(name);
    if let Expr::Str(_) = &*expr {
        Stmt::Str(Var { tipe: Type::Str, value: expr, name })
    } else {
        if let Expr::Id(_) = &*expr{
            Stmt::Str(Var { tipe: Type::Str, value: expr, name })
        }else{
            panic!("exp")
        }
    }
    
}

    }

 fn parse_primary_for_bianry(&mut self) -> *mut Expr<'src> {
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
  fn parse_expr(&mut self) ->*mut Expr<'src>{
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
        
    fn parse_term(&mut self) ->*mut Expr<'src>{
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
    fn parse_primary(& mut self) -> *mut Expr<'src> {
        let token = self.advance();
        let   expr= match token {
            Token::Number(val) => Expr::Num(val),
            Token::Identifier(name) => Expr::Id(name),
            Token::String(str) => Expr::Str(str),
            _ => panic!("error{:?}", token),
        };
       expr_add(self.arena, expr)
    }

  
}

pub fn ready_code<'src,'a>(arena: *mut Arena,code: &'src str,strint: &'a mut StringInterner<'src>)-> Vec<Stmt<'src>>{
      let tokens = tokenize(code);
      let mut parser = Parser::new(arena,strint,tokens);
      parser.parse()
}


