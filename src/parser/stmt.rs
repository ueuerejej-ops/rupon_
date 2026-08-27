use super::*;

impl<'src, 'arena> Parser<'src, 'arena> {
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

    pub(super) fn parse_call_for_expr(&mut self, name: &'src str) -> Funcall<'src> {
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

    pub(super) fn parse_stmt_for_func(&mut self) -> Stmt<'src> {
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
            Token::Loop => self.parse_loop(),
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

    pub(super) fn parse_statement(&mut self) -> Stmt<'src> {
        match self.current() {
            Token::Func => self.parse_func(),
            _ => {
                panic!("Expected string");
            }
        }
    }

    fn parse_loop(&mut self) -> Stmt<'src> {
        self.advance();

        if self.advance() != Token::Lcurly {
            panic!("expected lcurly")
        }

        let mut tokens = Vec::new();
        let mut depth = 0;

        loop {
            match self.current() {
                Token::Rcurly => {
                    if depth == 0 {
                        self.advance();
                        break;
                    }
                    depth -= 1;
                    tokens.push(self.current());
                    self.advance();
                }
                Token::Lcurly => {
                    depth += 1;
                    tokens.push(self.current());
                    self.advance();
                }
                _ => {
                    tokens.push(self.current());
                    self.advance();
                }
            }
        }

        tokens.push(Token::EOF);
        let mut parser = Parser::new(self.arena, tokens);
        parser.current_local = self.current_local.clone();
        parser.funcs = self.funcs.clone();
        parser.in_while = true;
        let stmts = parser.parse_for_func();
        Stmt::Loop(Loop {
            locals: parser.current_local,
            code: stmts,
        })
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

}
