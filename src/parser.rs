use crate::arena::Arena;
use crate::arena::expr_add;
use crate::parser::Type::Float;
use crate::token::Token;
use crate::token::tokenize;

#[derive(Debug, Clone, PartialEq)]
pub enum FunType {
    Str,
    Int,
    Bool,
    Char,
    Float,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param<'a> {
    pub name: &'a str,
    pub ty: Type,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Int,
    Char,
    Bool,
    Float,
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
    pub ty: Option<FunType>,
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
    Float(f64),
    Char(char),
    Id(&'a str),
    Bool(bool),
    Str(&'a str),
    Func(Funcall<'a>),
    None(Option<*mut Expr<'a>>),
    Binary(*mut Expr<'a>, BinaryOp, *mut Expr<'a>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct IfBlock<'a> {
    pub cond: Condition<'a>,
    pub code: Vec<Stmt<'a>>,
    pub locals: Vars<'a>,
    pub elsepart: Option<Vec<Stmt<'a>>>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct WhileBlock<'a> {
    pub cond: Condition<'a>,
    pub code: Vec<Stmt<'a>>,
    pub locals: Vars<'a>,
}
#[derive(Debug, PartialEq, Clone)]
pub enum CompareOp {
    Less,
    Greater,
    Equal,
    NotEqual,
}
#[derive(Debug, PartialEq, Clone)]
pub enum LogicalOp {
    And,
    Or,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Condition<'a> {
    Compare {
        left: *mut Expr<'a>,
        op: CompareOp,
        right: *mut Expr<'a>,
    },

    LogicalCompare {
        left: *mut Condition<'a>,
        op: LogicalOp,
        right: *mut Condition<'a>,
    },
    OnlyOne(*mut Expr<'a>),
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
    Char(Var<'a>),
    Float(Var<'a>),
    Str(Var<'a>),
    While(WhileBlock<'a>),
    Bool(Var<'a>),
    ReturnStmt(*mut Expr<'a>),
    Func(Func<'a>),
    Funcall(Funcall<'a>),
    Main(Func<'a>),
    If(IfBlock<'a>),
    Continue,
    Break,
}
#[derive(Debug, PartialEq)]
struct Parser<'src, 'arena> {
    arena: &'arena mut Arena<'src>,
    current_local: Vars<'src>,
    tokens: Vec<Token<'src>>,
    pos: usize,
    funcs: Vec<Func<'src>>,
    in_while: bool,
}

impl<'src> Vars<'src> {
    fn new() -> Self {
        Self { vars: Vec::new() }
    }

    fn save(&mut self, var: Var<'src>) {
        if !self.vars.iter().any(|v| v.clone().name == var.name) {
            self.vars.push(var);
        } else {
            panic!("error: variable `i` is already declared")
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
    fn new(arena: &'arena mut Arena<'src>, tokens: Vec<Token<'src>>) -> Self {
        Parser {
            arena,
            tokens,
            pos: 0,
            funcs: Vec::new(),
            current_local: Vars::new(),

            in_while: false,
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

    fn parse_call_for_expr(&mut self, name: &'src str) -> Funcall<'src> {
        let func = self
            .funcs
            .iter()
            .find(|f| f.name == name)
            .cloned()
            .expect("cannot find function");

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

        Funcall { name, args: params }
    }
    fn parse_if(&mut self) -> Stmt<'src> {
        self.advance();

        let mut contain_else = Option::None;
        let cond = self.parse_or();
        let mut code = Vec::new();

        self.advance();
        let mut depth = 0;
        loop {
            match self.current() {
                Token::Lcurly => {
                    depth += 1;
                    code.push(self.current());
                    self.advance();
                }

                Token::Rcurly => {
                    if depth == 0 {
                        self.advance();
                        break;
                    }
                    depth -= 1;
                    code.push(self.current());
                    self.advance();
                }

                _ => {
                    code.push(self.current());
                    self.advance();
                }
            }
        }

        if self.current() == Token::Else {
            let mut depth = 0;
            let mut code = Vec::new();
            self.advance();
            self.advance();
            loop {
                match self.current() {
                    Token::Lcurly => {
                        depth += 1;
                        code.push(self.current());
                        self.advance();
                    }

                    Token::Rcurly => {
                        if depth == 0 {
                            self.advance();
                            break;
                        }
                        depth -= 1;
                        code.push(self.current());
                        self.advance();
                    }
                    _ => {
                        code.push(self.current());
                        self.advance();
                    }
                }
            }
            code.push(Token::EOF);
            let mut parser = Parser::new(self.arena, code);
            parser.current_local = self.current_local.clone();
            parser.in_while = self.in_while;
            parser.funcs = self.funcs.clone();
            let stmts = parser.parse_for_func();
            contain_else = Some(stmts);
        }

        code.push(Token::EOF);

        let mut parser = Parser::new(self.arena, code);
        parser.current_local = self.current_local.clone();

        parser.in_while = self.in_while;
        parser.funcs = self.funcs.clone();
        let stmts = parser.parse_for_func();

        if let Some(else_code) = contain_else {
            Stmt::If(IfBlock {
                cond,
                code: stmts,
                locals: parser.current_local,
                elsepart: Some(else_code),
            })
        } else {
            Stmt::If(IfBlock {
                cond,
                code: stmts,
                locals: parser.current_local,
                elsepart: None,
            })
        }
    }

    fn parse_cond_atom(&mut self) -> Condition<'src> {
        let left = self.parse_expr();

        match self.current() {
            Token::Equal => {
                self.advance();
                let right = self.parse_expr();

                Condition::Compare {
                    left,
                    op: CompareOp::Equal,
                    right,
                }
            }

            Token::NotEqual => {
                self.advance();
                let right = self.parse_expr();

                Condition::Compare {
                    left,
                    op: CompareOp::NotEqual,
                    right,
                }
            }

            Token::Less => {
                self.advance();
                let right = self.parse_expr();

                Condition::Compare {
                    left,
                    op: CompareOp::Less,
                    right,
                }
            }

            Token::Greater => {
                self.advance();
                let right = self.parse_expr();

                Condition::Compare {
                    left,
                    op: CompareOp::Greater,
                    right,
                }
            }

            Token::Lcurly | Token::Or | Token::And => Condition::OnlyOne(left),

            token => panic!("EXPECTED CONDITION OPERATOR, GOT {:?}", token),
        }
    }

    fn parse_or(&mut self) -> Condition<'src> {
        let left = self.parse_and();

        match self.current() {
            Token::Or => {
                self.advance();

                let right = self.parse_or();

                let right_raw = self.arena.alloc(right.clone());
                let left_raw = self.arena.alloc(left.clone());
                println!("{:?}right", right.clone());
                println!("{:?}left", left.clone());
                Condition::LogicalCompare {
                    left: left_raw,
                    op: LogicalOp::Or,
                    right: right_raw,
                }
            }

            _ => left,
        }
    }

    fn parse_and(&mut self) -> Condition<'src> {
        let left = self.parse_cond_atom();

        match self.current() {
            Token::And => {
                self.advance();

                let right = self.parse_and();

                println!("{:?}right", right.clone());
                println!("{:?}left", left.clone());
                let right_raw = self.arena.alloc(right);
                let left_raw = self.arena.alloc(left);
                Condition::LogicalCompare {
                    left: left_raw,
                    op: LogicalOp::And,
                    right: right_raw,
                }
            }

            _ => left,
        }
    }

    fn parse_while(&mut self) -> Stmt<'src> {
        self.advance();
        let mut code = Vec::new();

        let cond = self.parse_and();
        self.advance();
        let mut depth = 0;

        loop {
            match self.current() {
                Token::Lcurly => {
                    depth += 1;
                    code.push(self.current());
                    self.advance();
                }

                Token::Rcurly => {
                    if depth == 0 {
                        self.advance();
                        break;
                    }
                    depth -= 1;
                    code.push(self.current());
                    self.advance();
                }
                _ => {
                    code.push(self.current());
                    self.advance();
                }
            }
        }

        code.push(Token::EOF);
        let mut parser = Parser::new(self.arena, code);
        parser.current_local = self.current_local.clone();
        parser.funcs = self.funcs.clone();
        parser.in_while = true;
        let stmts = parser.parse_for_func();
        parser.in_while = true;
        Stmt::While(WhileBlock {
            cond,
            code: stmts,
            locals: parser.current_local,
        })
    }
    fn do_break(&mut self) -> Stmt<'src> {
        if self.in_while != true {
            panic!("you can only call break in while")
        }
        self.advance();

        Stmt::Break
    }
    fn do_continue(&mut self) -> Stmt<'src> {
        if self.in_while != true {
            panic!("you can only call Continue in while")
        }
        self.advance();

        Stmt::Continue
    }
    fn parse_stmt_for_func(&mut self) -> Stmt<'src> {
        match self.current() {
            Token::Int => self.parse_int(),
            Token::Char => self.parse_char(),
            Token::Str => self.parse_str(),
            Token::Bool => self.parse_bool(),
            Token::Float => self.parse_float(),
            Token::Return => self.parse_return(),
            Token::Continue => self.do_continue(),
            Token::Func => self.parse_func(),
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::Break => self.do_break(),
            Token::Identifier(name) => {
                if self.next() == Token::Lparen {
                    self.parse_call(name)
                } else {
                    self.setvar(name)
                }
            }
            _ => {
                panic!("invalid token {:?}", self.current())
            }
        }
    }
    fn parse_statement(&mut self) -> Stmt<'src> {
        match self.current() {
            Token::Func => self.parse_func(),
            _ => {
                panic!("Expected string");
            }
        }
    }
    fn parse_return(&mut self) -> Stmt<'src> {
        self.advance();
        let expr = self.parse_expr();
        Stmt::ReturnStmt(expr)
    }
    fn parse_char(&mut self) -> Stmt<'src> {
        self.advance();
        let name = match self.advance() {
            Token::Identifier(name) => name,
            _ => panic!("expected name"),
        };

        if self.advance() != Token::Assign {
            panic!("expected '='")
        }
        let expr = self.check_expr(Type::Char);

        let var = Var {
            tipe: Type::Char,
            value: expr,
            name,
        };

        self.current_local.save(var.clone());
        Stmt::Char(var)
    }
    fn parse_float(&mut self) -> Stmt<'src> {
        self.advance();

        let name = match self.advance().clone() {
            Token::Identifier(name) => name,
            _ => panic!("Expected identifier"),
        };
        if self.advance() != Token::Assign {
            panic!("expected '='")
        }

        let expr = self.check_expr(Float);

        self.current_local.save(Var {
            tipe: Float,
            value: expr,
            name,
        });

        Stmt::Float(Var {
            tipe: Float,
            value: expr,
            name,
        })
    }
    fn parse_bool(&mut self) -> Stmt<'src> {
        self.advance();

        let name = match self.advance().clone() {
            Token::Identifier(name) => name,
            _ => panic!("Expected identifier"),
        };

        if self.advance() != Token::Assign {
            panic!("Expected '='");
        }

        let expr = self.check_expr(Type::Bool);

        self.current_local.save(Var {
            tipe: Type::Bool,
            value: expr,
            name,
        });
        Stmt::Bool(Var {
            tipe: Type::Bool,
            value: expr,
            name,
        })
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
                (Type::Int, Expr::Num(_)) => expr,
                (Type::Bool, Expr::Bool(_)) => expr,
                (Float, Expr::Float(_)) => expr,
                (Float, Expr::Id(name)) => {
                    let var = self.current_local.lookup_by_name(name);
                    if var.tipe == Float {
                        expr
                    } else {
                        panic!("invaild type")
                    }
                }
                (Type::Bool, Expr::Id(name)) => {
                    let var = self.current_local.lookup_by_name(name);

                    if var.tipe == Type::Bool {
                        expr
                    } else {
                        panic!("invalid type")
                    }
                }
                (Type::Bool, Expr::Func(func)) => {
                    let func = self.funcs.iter().find(|f| f.name == func.name).unwrap();
                    if func.ty == Some(FunType::Bool) {
                        expr
                    } else {
                        panic!("invaild type")
                    }
                }

                (Type::Int, Expr::Id(name)) => {
                    let var = self.current_local.lookup_by_name(name);
                    if var.tipe == Type::Int {
                        expr
                    } else {
                        panic!("invalid type")
                    }
                }
                (Float, Expr::Binary(_, _, _)) => expr,
                (Type::Int, Expr::Binary(_, _, _)) => expr,
                (Type::Int, Expr::Func(func)) => {
                    let func = self.funcs.iter().find(|f| f.name == func.name);
                    if func.unwrap().ty == Some(FunType::Int) {
                        expr
                    } else {
                        panic!("invalid type")
                    }
                }

                (Type::Str, Expr::Str(_)) => expr,
                (Type::Str, Expr::Id(name)) => {
                    let var = self.current_local.lookup_by_name(name);
                    if var.tipe == Type::Str {
                        expr
                    } else {
                        panic!("invalid type")
                    }
                }
                (Float, Expr::Func(func)) => {
                    let func = self.funcs.iter().find(|f| f.name == func.name);
                    if func.unwrap().ty == Some(FunType::Float) {
                        expr
                    } else {
                        panic!("inviled type")
                    }
                }

                (Type::Char, Expr::Func(func)) => {
                    let func = self.funcs.iter().find(|f| f.name == func.name);
                    if func.unwrap().ty == Some(FunType::Char) {
                        expr
                    } else {
                        panic!("inviled type")
                    }
                }
                (Type::Char, Expr::Char(_)) => expr,
                (Type::Char, Expr::Id(name)) => {
                    let var = self.current_local.lookup_by_name(name);
                    if var.tipe == Type::Char {
                        expr
                    } else {
                        panic!("invalid type")
                    }
                }
                (Type::Str, Expr::Func(func)) => {
                    let func = self.funcs.iter().find(|f| f.name == func.name);
                    if func.unwrap().ty == Some(FunType::Str) {
                        expr
                    } else {
                        panic!("invalid type")
                    }
                }

                _ => {
                    panic!("invalid type{:?}{:?}", &*expr, tipe)
                }
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

        let mut depth = 0;

        loop {
            match self.current() {
                Token::Lcurly => {
                    depth += 1;
                    code_token.push(self.current().clone());
                    self.advance();
                }
                Token::Rcurly => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    code_token.push(self.current().clone());
                    self.advance();
                }
                _ => {
                    code_token.push(self.current());
                    self.advance();
                }
            }
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
        let mut depth = 0;
        loop {
            match self.current() {
                Token::Lcurly => {
                    depth += 1;
                    code_token.push(self.current());
                    self.advance();
                }
                Token::Rcurly => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    code_token.push(self.current());
                    self.advance();
                }
                _ => {
                    code_token.push(self.current());
                    self.advance();
                }
            }
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
                        Some(Expr::Char(_)) => Some(FunType::Char),
                        Some(Expr::Bool(_)) => Some(FunType::Bool),
                        Some(Expr::Str(_)) => Some(FunType::Str),
                        Some(Expr::Binary(_, _, _)) => Some(FunType::Int),
                        Some(Expr::Func(ref func)) => {
                            let func = self.funcs.iter().find(|f| f.name == func.name).unwrap();
                            func.ty.clone()
                        }
                        Some(Expr::Id(name)) => {
                            let var = parser.current_local.lookup_by_name(name);
                            match var.tipe {
                                Type::Char => Some(FunType::Char),
                                Type::Int => Some(FunType::Int),
                                Type::Bool => Some(FunType::Bool),
                                Float => Some(FunType::Float),
                                Type::Str => Some(FunType::Str),
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

            Token::Float => Float,
            _ => panic!("Expected type "),
        }
    }
    fn setvar(&mut self, name: &'src str) -> Stmt<'src> {
        if self.current_local.are_here(name) {
            self.advance();
            if self.advance() != Token::Assign {
                panic!("Expected =")
            }
            let value = self.parse_expr();
            Stmt::Assign { name, value }
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

        let expr = self.check_expr(Type::Str);

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
    }

    fn parse_primary_for_bianry(&mut self) -> *mut Expr<'src> {
        let token = self.advance();

        match token {
            Token::Number(val) => expr_add(self.arena, Expr::Num(val)),
            Token::False => expr_add(self.arena, Expr::Bool(false)),
            Token::True => expr_add(self.arena, Expr::Bool(true)),
            Token::CharValue(val) => expr_add(self.arena, Expr::Char(val)),
            Token::FloatValue(val) => expr_add(self.arena, Expr::Float(val)),
            Token::Identifier(name) => {
                if self.current() == Token::Lparen {
                    self.advance();
                    let func = self.parse_call_for_expr(name);
                    self.arena.alloc_expr(Expr::Func(func))
                } else {
                    self.arena.alloc_expr(Expr::Id(name))
                }
            }

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

    fn get_type_out_expr(&mut self, expr: Expr<'src>) -> Type {
        match expr {
            Expr::Num(_) => Type::Int,
            Expr::Bool(_) => Type::Bool,
            Expr::Str(_) => Type::Str,
            Expr::Binary(_, _, _) => Type::Int,
            Expr::Float(_) => Float,
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
        name: "print",
        ty: None,
        returnv: None,
        locals: Vars { vars: vec![] },
    });
    parser.funcs.push(Func {
        args: vec![
            Param {
                name: "string1",
                ty: Type::Str,
            },
            Param {
                name: "string2",
                ty: Type::Str,
            },
        ],
        code: vec![],
        name: "str_cmp",
        ty: Some(FunType::Bool),
        returnv: None,
        locals: Vars { vars: vec![] },
    });

    println!("{:#?}", tokens);
    parser.parse()
}
