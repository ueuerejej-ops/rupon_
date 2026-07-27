use crate::arena::Arena;
use crate::arena::expr_add;
use crate::token::Token;
use crate::token::tokenize;
#[warn(unused)]
use core::panic;
#[derive(Debug, Clone, PartialEq)]
pub enum FunType<'src> {
    Str,
    Int,
    Id(&'src str),
}
#[derive(Debug, Clone, PartialEq)]
pub struct Param<'a> {
    pub name: &'a str,
    pub ty: Type,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Int,
    Str,
}
#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Func<'src> {
    pub args: Vec<Param<'src>>,
    pub code: Vec<Stmt<'src>>,
    pub name: &'src str,
    pub ty: Option<FunType<'src>>,
    pub returnv: Option<Expr<'src>>,
    pub locals: Vars<'src>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Funcall<'a> {
    pub name: &'a str,
    pub args: Vec<*mut Expr<'a>>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Expr<'a> {
    Num(i64),
    Id(&'a str),
    Str(&'a str),
    None(Option<*mut Expr<'a>>),
    Binary(*mut Expr<'a>, BinaryOp, *mut Expr<'a>),
}

#[derive(Debug, PartialEq, Clone)]

pub struct Vars<'src> {
    pub vars: Vec<Var<'src>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Var<'src> {
    pub tipe: Type,
    pub value: *mut Expr<'src>,
    pub name: &'src str,
}
#[derive(Debug, Clone, PartialEq)]

pub enum Stmt<'a> {
    Assign { name: &'a str, value: *mut Expr<'a> },
    Int(Var<'a>),

    Str(Var<'a>),

    ReturnStmt(*mut Expr<'a>),
    Func(Func<'a>),
    Funcall(Funcall<'a>),
    Expr(*mut Expr<'a>),
    Main(Func<'a>),
}
#[derive(Debug, PartialEq)]
struct Parser<'src, 'arena> {
    arena: &'arena mut Arena<'src>,
    current_local: Vars<'src>,
    tokens: Vec<Token<'src>>,
    pos: usize,
    funcs: Vec<Func<'src>>,
}

impl<'src> Vars<'src> {
    fn new() -> Self {
        Self { vars: Vec::new() }
    }

    fn save(&mut self, var: Var<'src>) {
        if !self.vars.iter().any(|v| v.clone() == var) {
            self.vars.push(var);
        }
    }
    fn lookup_by_name(&mut self, name: &'src str) -> Var<'src> {
        if let Some(var) = self.vars.iter().find(|v| v.name == name) {
            var.clone()
        } else {
            panic!()
        }
    }
    fn are_here(&mut self, name: &'src str) -> bool {
        if self.vars.iter().any(|v| v.name == name) {
            true
        } else {
            false
        }
    }
}

impl<'src, 'arena> Parser<'src, 'arena> {
    fn new(
        arena: &'arena mut Arena<'src>,
        // strint: &'a mut StringInterner<'src>,
        tokens: Vec<Token<'src>>,
    ) -> Self {
        Parser {
            arena: arena,
            tokens,
            pos: 0,
            funcs: Vec::new(),
            current_local: Vars::new(),
            // vars_name: strint,
        }
    }

    fn current(&self) -> Token<'src> {
        self.tokens[self.pos]
    }

    fn advance(&mut self) -> Token<'src> {
        let tok = self.tokens[self.pos].clone();
        self.pos += 1;
        tok
    }

    // fn previous(&self) -> Token<'src> {
    //     self.tokens[self.pos - 1]
    // }

    fn parse(&mut self) -> Vec<Stmt<'src>> {
        let mut statements = Vec::new();
        while self.current() != Token::EOF {
            statements.push(self.parse_statement());
        }

        statements
    }
    fn next(&mut self) -> Token<'src> {
        self.tokens[self.pos + 1]
    }
    fn parse_for_func(&mut self) -> Vec<Stmt<'src>> {
        let mut stmts = Vec::new();
        while self.current() != Token::EOF {
            stmts.push(self.parse_stmt_for_func());
        }
        stmts
    }
    fn parse_call(&mut self, name: &'src str) -> Stmt<'src> {
        let func = self
            .funcs
            .iter()
            .find(|f| f.name == name)
            .cloned()
            .expect("cannot find function");

        self.advance();
        self.advance();
        let mut params = Vec::new();
        while self.current() != Token::Rparen {
            params.push(self.parse_expr());
            if self.current() == Token::Comma {
                self.advance();
            }
        }
        if params.len() != func.args.len() {
            panic!("error args")
        }
        for (param_raw, arg) in params.iter().zip(func.args.iter()) {
            let param = unsafe { &**param_raw };

            if arg.ty != self.get_type_out_expr(param.clone()) {
                panic!("wrong argument type");
            }
        }
        self.advance();

        Stmt::Funcall(Funcall { name, args: params })
    }

    fn parse_stmt_for_func(&mut self) -> Stmt<'src> {
        match self.current() {
            Token::Int => self.parse_int(),
            Token::Str => self.parse_str(),
            Token::Return => self.parse_return(),
            Token::Func => self.parse_func(),
            Token::Identifier(name) => {
                if self.next() == Token::Lparen {
                    self.parse_call(name)
                } else {
                    self.setvar(name)
                }
            }
            _ => {
                let expr = self.parse_expr();

                unsafe {
                    if let Expr::Str(_) = &*expr {
                        Stmt::Expr(expr)
                    } else {
                        panic!("Expected string");
                    }
                }
            }
        }
    }
    fn parse_statement(&mut self) -> Stmt<'src> {
        match self.current() {
            Token::Func => self.parse_func(),
            _ => {
                let expr = self.parse_expr();

                unsafe {
                    if let Expr::Str(_) = &*expr {
                        Stmt::Expr(expr)
                    } else {
                        panic!("Expected string");
                    }
                }
            }
        }
    }
    fn parse_return(&mut self) -> Stmt<'src> {
        self.advance();
        let expr = self.parse_expr();

        Stmt::ReturnStmt(expr)
    }

    fn parse_int(&mut self) -> Stmt<'src> {
        self.advance();

        let name = match self.advance().clone() {
            Token::Identifier(name) => name,
            _ => panic!("Expected identifier"),
        };

        if self.advance() != Token::Assign {
            panic!("Expected '='");
        }

        let expr = self.check_expr(Type::Int);

        self.current_local.save(Var {
            tipe: Type::Int,
            value: expr,
            name,
        });
        Stmt::Int(Var {
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

    fn parse_main_func(&mut self) -> Stmt<'src> {
        let mut code_token: Vec<Token<'src>> = Vec::new();
        if self.current() != Token::Lparen && self.next() != Token::Rparen {
            panic!("cannot load args in main")
        }
        self.advance();
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

        let mut parser = Parser::new(self.arena, code_token);
        parser.funcs = self.funcs.clone();
        parser.current_local = Vars::new();
        let stmts = parser.parse_for_func();
        if stmts.iter().any(|stmt| matches!(stmt, Stmt::ReturnStmt(_))) {
            panic!("cannot return in main");
        }
        let func = Func {
            args: Vec::new(),
            code: stmts,
            name: "main",
            ty: None,
            returnv: None,
            locals: parser.current_local,
        };

        Stmt::Main(func)
    }
    fn parse_func(&mut self) -> Stmt<'src> {
        self.advance();

        let mut args: Vec<Param> = Vec::new();
        let mut code_token: Vec<Token<'src>> = Vec::new();
        let name = match self.advance() {
            Token::Identifier(name) => name,
            _ => panic!("Expected identifier"),
        };

        if name == "main" {
            return self.parse_main_func();
        }
        if self.advance() != Token::Lparen {
            panic!("Expected '('")
        }
        let mut locals = Vars::new();

        while self.current() != Token::Rparen {
            if let typee = self.parse_func_args() {
                let name = match self.advance() {
                    Token::Identifier(name) => name,
                    _ => panic!("Expected identifier"),
                };
                args.push(Param {
                    name,
                    ty: typee.clone(),
                });
                let expr = self.arena.alloc_expr(Expr::None(None));
                locals.save(Var {
                    tipe: typee,
                    value: expr,
                    name,
                });
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

        let mut parser = Parser::new(self.arena, code_token);
        parser.current_local = locals;
        parser.funcs = self.funcs.clone();
        let stmts = parser.parse_for_func();
        let mut returnv = None;

        let mut return_type: Option<FunType> = None;
        for stmt in &stmts {
            if let Stmt::ReturnStmt(value) = stmt {
                unsafe {
                    returnv = Some((*(*value)).clone());
                    return_type = match returnv {
                        Some(Expr::Num(_)) => Some(FunType::Int),
                        Some(Expr::Str(_)) => Some(FunType::Str),
                        Some(Expr::Binary(_, _, _)) => Some(FunType::Int),
                        Some(Expr::Id(name)) => {
                            if parser.current_local.are_here(name) {
                                Some(FunType::Id(name))
                            } else {
                                panic!()
                            }
                        }
                        _ => panic!("sd"),
                    }
                }
                break;
            }
        }

        self.funcs.push(Func {
            args: args.clone(),
            code: stmts.clone(),
            name,
            ty: return_type.clone(),
            returnv: returnv.clone(),
            locals: parser.current_local.clone(),
        });
        Stmt::Func(Func {
            args,
            code: stmts,
            name,
            returnv,
            ty: return_type,
            locals: parser.current_local,
        })
    }
    fn parse_func_args(&mut self) -> Type {
        match self.advance().clone() {
            Token::Int => Type::Int,
            Token::Str => Type::Str,

            _ => panic!("Expected type "),
        }
    }
    fn setvar(&mut self, name: &'src str) -> Stmt<'src> {
        if self.current_local.are_here(name) {
            self.advance();
            if self.advance() != Token::Assign {
                panic!("Expected =")
            }
            let value = self.parse_primary();
            Stmt::Assign { name, value: value }
        } else {
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
            if let Expr::Str(_) = &*expr {
                self.current_local.save(Var {
                    tipe: Type::Str,
                    value: expr,
                    name,
                });
                Stmt::Str(Var {
                    tipe: Type::Str,
                    value: expr,
                    name,
                })
            } else {
                if let Expr::Id(_) = &*expr {
                    self.current_local.save(Var {
                        tipe: Type::Str,
                        value: expr,
                        name,
                    });
                    Stmt::Str(Var {
                        tipe: Type::Str,
                        value: expr,
                        name,
                    })
                } else {
                    panic!("exp")
                }
            }
        }
    }

    fn parse_primary_for_bianry(&mut self) -> *mut Expr<'src> {
        let token = self.advance();

        match token {
            Token::Number(val) => expr_add(self.arena, Expr::Num(val)),

            Token::Identifier(name) => expr_add(self.arena, Expr::Id(name)),

            Token::String(str) => expr_add(self.arena, Expr::Str(str)),

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
    fn parse_expr(&mut self) -> *mut Expr<'src> {
        let mut left = self.parse_term();
        while matches!(self.current(), Token::Mines | Token::Add) {
            let op = match self.advance() {
                Token::Add => BinaryOp::Add,
                Token::Mines => BinaryOp::Sub,
                _ => unreachable!(),
            };

            let right = self.parse_term();

            let expr = Expr::Binary(left, op, right);
            left = expr_add(self.arena, expr);
        }
        left
    }

    fn parse_term(&mut self) -> *mut Expr<'src> {
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
    fn parse_primary(&mut self) -> *mut Expr<'src> {
        let token = self.advance();
        let expr = match token {
            Token::Number(val) => Expr::Num(val),
            Token::Identifier(name) => Expr::Id(name),
            Token::String(str) => Expr::Str(str),
            _ => panic!("error{:?}", token),
        };
        expr_add(self.arena, expr)
    }
    fn get_type_out_expr(&mut self, expr: Expr<'src>) -> Type {
        match expr {
            Expr::Num(_) => Type::Int,
            Expr::Str(_) => Type::Str,
            Expr::Binary(_, _, _) => Type::Int,
            Expr::Id(name) => {
                let var = self.current_local.lookup_by_name(name);
                var.tipe
            }

            _ => panic!(),
        }
    }
}

pub fn ready_code<'src, 'a, 'arena>(
    arena: &'arena mut Arena<'src>,
    code: &'src str,
) -> Vec<Stmt<'src>> {
    let tokens = tokenize(code);
    let mut parser = Parser::new(arena, tokens.clone());
    parser.funcs.push(Func {
        args: vec![Param {
            name: "string",
            ty: Type::Str,
        }],
        code: vec![],
        name: "puts",
        ty: None,
        returnv: None,
        locals: Vars { vars: vec![] },
    });
    parser.parse()
}
