#[derive(Debug, PartialEq, Clone, Copy)]

pub enum Token<'src> {
    Func,
    Return,
    Rparen,
    Lparen,
    Lcurly,
    Rcurly,
    Comma,
    Assign,
    Mines,
    Float,
    Add,
    Char,
    Mul,
    Loop,
    Div,
    Identifier(&'src str),
    Number(i64),
    String(&'src str),
    Int,
    CharValue(char),
    Str,
    Else,
    While,
    And,
    Break,
    Continue,
    Or,
    If,
    FloatValue(f64),
    Less,
    Greater,
    Equal,
    NotEqual,
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
                if bytes[i + 1] == b'=' {
                    tokens.push(Token::Equal);
                    i += 2;
                    continue;
                } else {
                    tokens.push(Token::Assign);
                    i += 1;
                    continue;
                }
            }

            b'>' => {
                tokens.push(Token::Greater);
                i += 1;
                continue;
            }
            b'<' => {
                tokens.push(Token::Less);
                i += 1;
                continue;
            }
            b'+' => {
                tokens.push(Token::Add);
                i += 1;
                continue;
            }

            b'&' => {
                if bytes[i + 1] == b'&' {
                    tokens.push(Token::And);
                    i += 2;
                    continue;
                }
            }

            b'|' => {
                if bytes[i + 1] == b'|' {
                    tokens.push(Token::Or);
                    i += 2;
                    continue;
                }
            }

            b'!' => {
                if bytes[i + 1] == b'=' {
                    tokens.push(Token::NotEqual);
                    i += 2;
                    continue;
                } else {
                    panic!("expected '='")
                }
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
            let mut dots = false;
            while i < bytes.len() {
                if bytes[i].is_ascii_digit() {
                    i += 1
                } else if bytes[i] == b'.' && !dots {
                    dots = true;
                    i += 1
                } else {
                    break;
                }
            }
            let num_str = &code[start..i];
            if dots {
                let num = num_str.parse::<f64>().unwrap();
                tokens.push(Token::FloatValue(num))
            } else {
                let num = num_str.parse::<i64>().unwrap();
                tokens.push(Token::Number(num))
            }
            continue;
        }

        if bytes[i] == b'\'' {
            if !bytes[i + 1].is_ascii_digit() && !bytes[i + 1].is_ascii_alphabetic() {
                panic!("expected char")
            }
            if bytes[i + 2] != b'\'' {
                panic!("Expected '")
            }
            let char = bytes[i + 1] as char;
            tokens.push(Token::CharValue(char));

            i += 3;
            continue;
        }
        if bytes[i] == b'"' {
            i += 1;

            let start = i;
            while i < bytes.len() && bytes[i] != b'"' {
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
                "float" => tokens.push(Token::Float),
                "func" => tokens.push(Token::Func),
                "if" => tokens.push(Token::If),
                "while" => tokens.push(Token::While),
                "continue" => tokens.push(Token::Continue),
                "else" => tokens.push(Token::Else),
                "loop" => tokens.push(Token::Loop),
                "break" => tokens.push(Token::Break),
                "char" => tokens.push(Token::Char),
                _ => tokens.push(Token::Identifier(ident_str)),
            }
            continue;
        }
    }
    tokens.push(Token::EOF);
    tokens
}
