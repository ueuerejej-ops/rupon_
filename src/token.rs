
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
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

    Identifier(String),
    Number(i64),
    String(String),
    Int,
    Str,

    EOF,
}
pub fn tokenize(code: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();

    let chars: Vec<char> = code.chars().collect();

    let mut i = 0;

    while i < chars.len() {
        let ch: char = chars[i];
        if ch.is_whitespace() {
            i += 1;
            continue;
        }

        match ch {
            '=' => {
                tokens.push(Token::Assign);
                i += 1;
                continue;
            }

            '+' => {
                tokens.push(Token::Add);
                i += 1;
                continue;
            }
            '-' => {
                tokens.push(Token::Mines);
                i += 1;
                continue;
            }
            '*' => {
                tokens.push(Token::Mul);
                i += 1;
                continue;
            }
            '/' => {
                tokens.push(Token::Div);
                i += 1;
                continue;
            }
            ')' => {
                tokens.push(Token::Rparen);
                i += 1;
                continue;
            }
            '}' => {
                tokens.push(Token::Rcurly);
                i += 1;
                continue;
            }
            ',' => {
                tokens.push(Token::Comma);
                i += 1;
                continue;
            }
            '{' => {
                tokens.push(Token::Lcurly);
                i += 1;
                continue;
            }
            '(' => {
                tokens.push(Token::Lparen);
                i += 1;
                continue;
            }
            _ => {}
        }

        if ch.is_ascii_digit() {
            let mut num_str = String::new();

            while i < chars.len() && chars[i].is_ascii_digit() {
                num_str.push(chars[i]);
                i += 1
            }
            let num = num_str.parse::<i64>().unwrap();
            tokens.push(Token::Number(num));
            continue;
        }

        if ch == '"' {
            i += 1;
            let mut str = String::new();
            while chars[i] != '"' && i < chars.len() {
                str.push(chars[i]);
                i += 1;
            }

            tokens.push(Token::String(str));
            i += 1;

            continue;
        }

        if ch.is_alphabetic() || ch == '_' && ch != '"' {
            let mut ident_str = String::new();
            while i < chars.len() && (chars[i].is_alphabetic() || chars[i] == '_') {
                ident_str.push(chars[i]);
                i += 1;
            }

            match ident_str.as_str() {
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
