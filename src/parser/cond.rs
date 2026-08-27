use super::*;

impl<'src, 'arena> Parser<'src, 'arena> {
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

    pub(super) fn parse_or(&mut self) -> Condition<'src> {
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

    pub(super) fn parse_and(&mut self) -> Condition<'src> {
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

}
