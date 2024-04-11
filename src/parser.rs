use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::tokenizer::{Token, TokenType, Literal};

/*
PROGRAM
program        → declaration* EOF ;

DECLARATIONS
declaration    → varDecl
               | statement ;
varDecl        → "var" IDENTIFIER ( "=" expression )? ";" ;

STATEMENTS
statement      → exprStmt
               | printStmt ;
exprStmt       → expression ";" ;
printStmt      → "print" expression ";" ;

EXPRESSIONS
expression → equality ;
equality → comparison ( ( "!=" | "==" ) comparison )* ;
comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term → factor ( ( "-" | "+" ) factor )* ;
factor → unary ( ( "/" | "*" ) unary )* ;
unary → ( "!" | "-" ) unary
| primary ;
primary → NUMBER | STRING | "true" | "false" | "nil"
| "(" expression ")" ;
*/

#[derive(Debug)]
pub enum ParserError {
    SyntaxError(Token, String)
}

// Tokenizer state represents the state of the tokenizer function as it progresses
#[derive(Debug)]
struct ParserState {
    pub tokens: Vec<Token>,
    pub current: usize
}

impl ParserState {
    fn new(tokens: Vec<Token>) -> ParserState {
        ParserState {
            tokens: tokens,
            current: 0,
        }
    }

    fn advance_if_next(&mut self, types: &[TokenType]) -> bool {
        for ty in types {
            if self.check(ty.clone()) {
                self.advance();
                return true;
            }
        }
        return false;
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.at_end() { return false; }
        return self.peek().token_type == token_type;
    }

    fn advance(&mut self) -> &Token{
        if !self.at_end() { self.current += 1}
        return self.previous();
    }

    fn at_end(&self) -> bool {
        return self.peek().token_type == TokenType::Eof;
    }

    fn peek(&self) -> &Token {
        return &self.tokens[self.current];
    }

    fn previous(&self) -> &Token{
        return &self.tokens[self.current-1];
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<&Token, ParserError> {
        if self.check(token_type) { return Ok(self.advance()); }
        Err(ParserError::SyntaxError(self.peek().clone(), message.to_string()))
    }
}

fn parse_expression(state: &mut ParserState) -> Result<Expr, ParserError> { 
    fn expression(state: &mut ParserState) -> Result<Expr, ParserError> {
        Ok(equality(state)?)
    }

    fn equality(state: &mut ParserState) -> Result<Expr, ParserError> {
        let mut expr = comparison(state)?;
        while state.advance_if_next(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = state.previous().clone();
            let right = comparison(state)?;
            expr = Expr::Binary { lhs: Box::new(expr), op: operator, rhs: Box::new(right) };
        }
        Ok(expr)
    }

    fn comparison(state: &mut ParserState) -> Result<Expr, ParserError> {
        let mut expr = term(state)?;
        while state.advance_if_next(&[TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual]) {
            let operator = state.previous().clone();
            let right = term(state)?;
            expr = Expr::Binary { lhs: Box::new(expr), op: operator, rhs: Box::new(right) }
        }
        Ok(expr)
    }

    fn term(state: &mut ParserState) -> Result<Expr, ParserError> {
        let mut expr = factor(state)?;
        while state.advance_if_next(&[TokenType::Minus, TokenType::Plus]) {
            let operator = state.previous().clone();
            let right = factor(state)?;
            expr = Expr::Binary { lhs: Box::new(expr), op: operator, rhs: Box::new(right) };
        }
        Ok(expr)
    }

    fn factor(state: &mut ParserState) -> Result<Expr, ParserError> {
        let mut expr = unary(state)?;
        while state.advance_if_next(&[TokenType::Slash, TokenType::Star]) {
            let operator = state.previous().clone();
            let right = unary(state)?;
            expr = Expr::Binary { lhs: Box::new(expr), op: operator, rhs: Box::new(right) }
        }
        Ok(expr)
    }

    fn unary(state: &mut ParserState) -> Result<Expr, ParserError> {
        if state.advance_if_next(&[TokenType::Bang, TokenType::Minus]) {
            let operator = state.previous().clone();
            let right = unary(state)?;
            return Ok(Expr::Unary { op: operator, rhs: Box::new(right) });
        }
        Ok(primary(state)?)
    }

    fn primary(state: &mut ParserState) -> Result<Expr, ParserError> {;
        match state.peek().token_type.clone() {
            TokenType::Literal(literal) => {
                match literal {
                    Literal::Identifier(identifier) => {
                        state.advance();
                        return Ok(Expr::Variable { name: state.previous().clone() });
                    }
                    _ => {
                        state.advance();
                        return Ok(Expr::Literal { literal: literal.clone()});
                    }
                }
            }
            TokenType::LeftParenthesis => {
                state.advance();
                let expr = expression(state)?;
                state.consume(TokenType::RightParenthesis, "Expect ')' after expression.")?;
                return Ok(Expr::Grouping{expr: Box::new(expr)});
            }
            _ => Err(ParserError::SyntaxError(state.peek().clone(), String::from("Expected expression")))
        }

    }  

    Ok(expression(state)?)
}

fn parse_statement(state: &mut ParserState) -> Result<Stmt, ParserError> {
    fn print_statement(state: &mut ParserState) -> Result<Stmt, ParserError> {
        let value = parse_expression(state)?;
        state.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print { expr: value })
    }
    
    fn expression_statement(state: &mut ParserState) -> Result<Stmt, ParserError> {
        let value = parse_expression(state)?;
        state.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Expression { expr: value })
    }

    if state.advance_if_next(&[TokenType::Print]) {
        return print_statement(state);
    } else {
        return expression_statement(state);
    }
}

fn parse_declaration(state: &mut ParserState) -> Result<Stmt, ParserError> {
    fn var_declaration(state: &mut ParserState) -> Result<Stmt, ParserError> {
        let name = if let TokenType::Literal(Literal::Identifier(_)) = state.peek().token_type {
            state.peek().clone()
        } else {
            return Err(ParserError::SyntaxError(state.peek().clone(), String::from("Expected variable name.")));
        };
        state.advance();

        let initializer;
        if state.advance_if_next(&[TokenType::Equal]) {
            initializer = parse_expression(state)?;
        } else {
            initializer = Expr::Literal { literal: Literal::Nil };
        }

        state.consume(TokenType::Semicolon, "Expected ';' after variable declaration")?;
        Ok(Stmt::Var { token: name, intializer: initializer })
    }

    if state.advance_if_next(&[TokenType::Var]) {
        var_declaration(state)
    } else {
        parse_statement(state)
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Stmt>, ParserError> {
    let mut state = ParserState::new(tokens);
    let mut statements = Vec::new();

    while !state.at_end() {
        statements.push(parse_declaration(&mut state)?)
    }
    Ok(statements)
}