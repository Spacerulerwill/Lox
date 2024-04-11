use std::iter::Peekable;
use std::str::Chars;
use std::fmt;
use unicode_width::UnicodeWidthChar;

// Literal types
#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Identifier(String),
    String(String),
    Number(f64),
    Boolean(bool),
    Nil
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Identifier(indentifier) => write!(f, "{}", indentifier),
            Literal::String(string) => write!(f, "{}", string),
            Literal::Number(val) => write!(f, "{}", val),
            Literal::Boolean(val) => write!(f, "{}", val),
            Literal::Nil => write!(f, "nil"),
        }
    }
}

// Token types
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParenthesis,
    RightParenthesis,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals.
    Literal(Literal),
    // Keywords.
    And,
    Class,
    Else,
    Fun,
    For,
    If,
    Or,
    Print,
    Return,
    Super,
    This,
    Var,
    While,
    // End of file
    Eof,
}

// Error types
#[derive(Debug)]
pub enum TokenizerError {
    BadChar(char, usize, usize),
    UnterminatedStringLiteral(usize, usize),
    UnterminatedBlockComment(usize, usize),
    InvalidNumberLiteral(String, usize, usize),
}

// A token consists of a type, a lexeme (string representing the token), line and column number
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub col: usize,
}

// Tokenizer state represents the state of the tokenizer function as it progresses
#[derive(Debug)]
struct TokenizerState<'a> {
    tokens: Vec<Token>,
    iter: Peekable<Chars<'a>>,
    line: usize,
    col: usize,
}

impl<'a> TokenizerState<'a> {
    fn new(source: &'a str) -> TokenizerState<'a> {
        TokenizerState {
            tokens: Vec::new(),
            iter: source.chars().peekable(),
            line: 1,
            col: 1,
        }
    }
}

pub fn tokenize(source: &String) -> Result<Vec<Token>, TokenizerError> {
    let mut state = TokenizerState::new(source.trim());

    // Add a token to the token Vec, increase column, count, advance iterator
    fn add_token(state: &mut TokenizerState, token_type: TokenType, lexeme: String) {
        let mut new_col = state.col;
        let mut new_line = state.line;
        for char in lexeme.chars() {
            if char != '\n' && UnicodeWidthChar::width(char).unwrap_or(0) == 0 {
                continue;
            }
            match char {
                '\n' => {
                    new_col = 1;
                    new_line += 1;
                }
                '\t' => new_col += 4,
                _ => new_col += 1,
            }
        }
        state.tokens.push(Token {
            token_type: token_type,
            lexeme: lexeme,
            line: state.line,
            col: state.col,
        });
        state.line = new_line;
        state.col = new_col;
    }

    // Return true if the next char is what we want it to be
    fn is_next<F>(state: &mut TokenizerState, condition: F) -> bool
    where
        F: FnOnce(char) -> bool,
    {
        match state.iter.peek() {
            Some(&ch) => return condition(ch),
            None => return false,
        }
    }

    // Tokenize 2 char tokens (!=, ==, <=, >=)
    fn tokenize_double_char_token(
        state: &mut TokenizerState,
        single_char_token_type: TokenType,
        double_char_token_type: TokenType,
        first: char,
        second: char,
    ) {
        if is_next(state, |char| char == second) {
            add_token(
                state,
                double_char_token_type,
                String::from(first.to_string() + &second.to_string()),
            );
            state.iter.next();
        } else {
            add_token(state, single_char_token_type, first.to_string())
        };
    }

    // Tokenize the slash character - can result in a slash token, a line comment or a block comment
    fn tokenize_slash(state: &mut TokenizerState) -> Result<(), TokenizerError> {
        // Line comment
        if is_next(state, |char| char == '/') {
            while match state.iter.peek() {
                Some(&c) => c != '\n',
                _ => false,
            } {
                state.iter.next();
            }
        }
        // Block comment
        else if is_next(state, |char| char == '*') {
            loop {
                if let Some(ch) = state.iter.next() {
                    if ch == '*' {
                        if let Some(&next) = state.iter.peek() {
                            if next == '/' {
                                state.iter.next();
                                break;
                            }
                        } else {
                            return Err(TokenizerError::UnterminatedBlockComment(
                                state.line, state.col,
                            ));
                        }
                    }
                } else {
                    return Err(TokenizerError::UnterminatedBlockComment(
                        state.line, state.col,
                    ));
                }
            }
        }
        // Slash
        else {
            add_token(state, TokenType::Slash, '/'.to_string())
        }
        Ok(())
    }

    // Tokenize a string literal
    fn tokenize_string_literal(state: &mut TokenizerState) -> Result<(), TokenizerError> {
        // String literals
        let mut val = String::new();
        loop {
            match state.iter.peek() {
                Some(&c) => {
                    if c == '"' {
                        break;
                    }
                    val.push(c);
                    state.iter.next();
                }
                None => {
                    return Err(TokenizerError::UnterminatedStringLiteral(
                        state.line, state.col,
                    ));
                }
            }
        }
        let lexeme = format!("\"{}\"", &val);
        add_token(state, TokenType::Literal(Literal::String(val)), lexeme);
        state.iter.next();
        Ok(())
    }

    // Tokenize a number literal
    fn tokenize_number_literal(
        state: &mut TokenizerState,
        first_char: char,
    ) -> Result<(), TokenizerError> {
        let mut lexeme = first_char.to_string();

        // Get number before decimal point
        while let Some(&c) = state.iter.peek() {
            if !c.is_digit(10) {
                break;
            }

            lexeme.push(c);
            state.iter.next();
        }

        // Is there a decimal point?
        if is_next(state, |char| char == '.') {
            state.iter.next();
            lexeme.push('.');
        // No decimal point, just add token
        } else {
            add_token(
                state,
                TokenType::Literal(Literal::Number(lexeme.parse().unwrap())),
                lexeme,
            );
            return Ok(());
        }

        // Check there a non digit next. If there is no digit return an error (12. is invalid)
        if !is_next(state, |char| char.is_digit(10)) {
            return Err(TokenizerError::InvalidNumberLiteral(
                lexeme, state.line, state.col,
            ));
        }

        // Get rest of digits after decimal point
        while let Some(&c) = state.iter.peek() {
            if !c.is_digit(10) {
                break;
            }
            lexeme.push(c);
            state.iter.next();
        }

        add_token(
            state,
            TokenType::Literal(Literal::Number(lexeme.parse().unwrap())),
            lexeme,
        );
        Ok(())
    }

    // Tokenize an alphabetical character string into either an identifier or a keyword
    fn tokenize_alphabetical(
        state: &mut TokenizerState,
        first_char: char,
    ) -> Result<(), TokenizerError> {
        let mut lexeme = first_char.to_string();
        while let Some(&c) = state.iter.peek() {
            if !(c.is_alphabetic() || c == '_') {
                break;
            }
            lexeme.push(c);
            state.iter.next();
        }

        if let Some(token_type) = get_reserved_word(&lexeme) {
            add_token(state, token_type, lexeme);
        } else {
            add_token(
                state,
                TokenType::Literal(Literal::Identifier(lexeme.clone())),
                lexeme,
            );
        }
        Ok(())
    }

    // Reserved word tokens
    fn get_reserved_word(lexeme: &str) -> Option<TokenType> {
        match lexeme {
            "and" => Some(TokenType::And),
            "class" => Some(TokenType::Class),
            "else" => Some(TokenType::Else),
            "false" => Some(TokenType::Literal(Literal::Boolean(false))),
            "fun" => Some(TokenType::Fun),
            "for" => Some(TokenType::For),
            "if" => Some(TokenType::If),
            "nil" => Some(TokenType::Literal(Literal::Nil)),
            "or" => Some(TokenType::Or),
            "print" => Some(TokenType::Print),
            "return" => Some(TokenType::Return),
            "super" => Some(TokenType::Super),
            "this" => Some(TokenType::This),
            "true" => Some(TokenType::Literal(Literal::Boolean(true))),
            "var" => Some(TokenType::Var),
            "while" => Some(TokenType::While),
            _ => None,
        }
    }

    // Tokenize
    while let Some(ch) = state.iter.next() {
        // If it's not a newline char, but has 0 width we skip
        if ch != '\n' && UnicodeWidthChar::width(ch).unwrap_or(0) == 0 {
            continue;
        }
        // Otherwise match on the character and perform actions as neccessary
        match ch {
            '\n' => {
                state.line += 1;
                state.col = 1;
            }
            '\t' => state.col += 4,
            ' ' => state.col += 1,
            '(' => add_token(&mut state, TokenType::LeftParenthesis, ch.to_string()),
            ')' => add_token(&mut state, TokenType::RightParenthesis, ch.to_string()),
            '{' => add_token(&mut state, TokenType::LeftBrace, ch.to_string()),
            '}' => add_token(&mut state, TokenType::RightBrace, ch.to_string()),
            ',' => add_token(&mut state, TokenType::Comma, ch.to_string()),
            '.' => add_token(&mut state, TokenType::Dot, ch.to_string()),
            '-' => add_token(&mut state, TokenType::Minus, ch.to_string()),
            '+' => add_token(&mut state, TokenType::Plus, ch.to_string()),
            ';' => add_token(&mut state, TokenType::Semicolon, ch.to_string()),
            '/' => tokenize_slash(&mut state)?,
            '*' => add_token(&mut state, TokenType::Star, ch.to_string()),
            '!' => tokenize_double_char_token(
                &mut state,
                TokenType::Bang,
                TokenType::BangEqual,
                ch,
                '=',
            ),
            '=' => tokenize_double_char_token(
                &mut state,
                TokenType::Equal,
                TokenType::EqualEqual,
                ch,
                '=',
            ),
            '<' => tokenize_double_char_token(
                &mut state,
                TokenType::Less,
                TokenType::LessEqual,
                ch,
                '=',
            ),
            '>' => tokenize_double_char_token(
                &mut state,
                TokenType::Greater,
                TokenType::GreaterEqual,
                ch,
                '=',
            ),
            '"' => tokenize_string_literal(&mut state)?,
            '0'..='9' => tokenize_number_literal(&mut state, ch)?,
            '_' | 'a'..='z' | 'A'..='Z' => tokenize_alphabetical(&mut state, ch)?,
            _ => return Err(TokenizerError::BadChar(ch, state.line, state.col)),
        }
    }
    add_token(&mut state, TokenType::Eof, "".to_string());
    Ok(state.tokens)
}
