use super::*;

impl<'src, 'arena> Parser<'src, 'arena> {
    pub(super) fn check_expr(&mut self, tipe: Type) -> *mut Expr<'src> {
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

    pub(super) fn parse_expr(&mut self) -> *mut Expr<'src> {
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

    pub(super) fn get_type_out_expr(&mut self, expr: Expr<'src>) -> Type {
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
