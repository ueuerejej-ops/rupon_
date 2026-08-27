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
pub struct Loop<'src> {
    pub locals: Vars<'src>,
    pub code: Vec<Stmt<'src>>,
}
#[derive(Debug, Clone, PartialEq)]

pub enum Stmt<'a> {
    Assign { name: &'a str, value: *mut Expr<'a> },
    Int(Var<'a>),
    Char(Var<'a>),
    Loop(Loop<'a>),
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


mod stmt;
mod cond;
mod expr;

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
