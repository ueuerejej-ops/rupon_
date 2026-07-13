#[derive(Debug, PartialEq, Clone, Copy)]

pub enum Token<'src> {
    Call,
    Func,
    Return,
    Rparen,
    Lparen,
    Lcurly,
    Rcurly,
    Comma,
    Assign,
    Mines,
    Add,
    Mul,
    Div,

    Identifier(&'src str),
    Number(i64),
    String(&'src str),
    Int,
    Str,

    EOF,
}
pub fn tokenize<'src>(code: &'src str) -> Vec<Token<'src>> {
    let mut tokens: Vec<Token> = Vec::new();

    let bytes = code.as_bytes();

    let mut i = 0;

    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }

        match bytes[i] {
            b'=' => {
                tokens.push(Token::Assign);
                i += 1;
                continue;
            }

            b'+' => {
                tokens.push(Token::Add);
                i += 1;
                continue;
            }

            b'-' => {
                tokens.push(Token::Mines);
                i += 1;
                continue;
            }

            b'*' => {
                tokens.push(Token::Mul);
                i += 1;
                continue;
            }

            b'/' => {
                tokens.push(Token::Div);
                i += 1;
                continue;
            }

            b')' => {
                tokens.push(Token::Rparen);
                i += 1;
                continue;
            }

            b'}' => {
                tokens.push(Token::Rcurly);
                i += 1;
                continue;
            }

            b',' => {
                tokens.push(Token::Comma);
                i += 1;
                continue;
            }

            b'{' => {
                tokens.push(Token::Lcurly);
                i += 1;
                continue;
            }

            b'(' => {
                tokens.push(Token::Lparen);
                i += 1;
                continue;
            }

            _ => {}
        }

        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1
            }
            let num = code[start..i].parse::<i64>();
            tokens.push(Token::Number(num.unwrap()));
            continue;
        }

        if bytes[i] == b'"' {
            i += 1;

            let start = i;
            while  i < bytes.len() &&bytes[i] != b'"' {
                i += 1;
            }
            let text = &code[start..i];
            
            tokens.push(Token::String(text));
            i += 1;

            continue;
        }

        if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' && bytes[i] != b'"' {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphabetic() || bytes[i] == b'_') {
                i += 1;
            }
            let ident_str = &code[start..i];
            match ident_str {
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
