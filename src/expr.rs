use std::collections::HashMap;

use crate::{tokenizer::{Literal, Token, TokenType}, InterpreterState};

#[derive(Debug, PartialEq)]
pub enum Expr {
    Binary {
        lhs: Box<Expr>,
        op: Token,
        rhs: Box<Expr>
    },
    Unary {
        op: Token,
        rhs: Box<Expr>
    },
    Grouping {
        expr: Box<Expr>
    },
    Literal {
        literal: Literal
    },
    Variable {
        name: Token
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    TypeError(Token, String),
    NameError(Token)
}

impl Expr {
    pub fn evaluate(self, interpreter_state: &mut InterpreterState) -> Result<Literal, RuntimeError> {
        match self {
            Expr::Binary { lhs, op, rhs } => {
                let op1 = lhs.evaluate(interpreter_state)?;
                let op2 = rhs.evaluate(interpreter_state)?;
                match op.token_type {
                    TokenType::Plus => {
                        match (op1, op2) {
                            (Literal::Number(num1), Literal::Number(num2)) => Ok(Literal::Number(num1 + num2)),
                            (Literal::String(str1), Literal::String(str2)) => Ok(Literal::String(str1 + &str2)),
                            _ => Err(RuntimeError::TypeError(op.clone(), String::from(format!("Unsupported operand types for '{}'", &op.lexeme))))
                        }
                    }
                    TokenType::Minus => {
                        match (op1, op2) {
                            (Literal::Number(num1), Literal::Number(num2)) => Ok(Literal::Number(num1 - num2)),
                            _ => Err(RuntimeError::TypeError(op.clone(), String::from(format!("Unsupported operand types for '{}'", &op.lexeme))))
                        }
                    }
                    TokenType::Star => {
                        match (op1, op2) {
                            (Literal::Number(num1), Literal::Number(num2)) => Ok(Literal::Number(num1 * num2)),
                            _ => Err(RuntimeError::TypeError(op.clone(), String::from(format!("Unsupported operand types for '{}'", &op.lexeme))))
                        }
                    }
                    TokenType::Slash => {
                        match (op1, op2) {
                            (Literal::Number(num1), Literal::Number(num2)) => Ok(Literal::Number(num1 / num2)),
                            _ => Err(RuntimeError::TypeError(op.clone(), String::from(format!("Unsupported operand types for '{}'", &op.lexeme))))
                        }
                    }
                    TokenType::EqualEqual => {
                        Ok(Literal::Boolean(op1 == op2))
                    }
                    TokenType::BangEqual => {
                        Ok(Literal::Boolean(op1 != op2))
                    }
                    TokenType::Greater => {
                        match (op1, op2) {
                            (Literal::Number(num1), Literal::Number(num2)) => Ok(Literal::Boolean(num1 > num2)),
                            _ => Err(RuntimeError::TypeError(op.clone(), String::from(format!("Unsupported operand types for '{}'", &op.lexeme))))
                        }
                    }
                    TokenType::GreaterEqual => {
                        match (op1, op2) {
                            (Literal::Number(num1), Literal::Number(num2)) => Ok(Literal::Boolean(num1 >= num2)),
                            _ => Err(RuntimeError::TypeError(op.clone(), String::from(format!("Unsupported operand types for '{}'", &op.lexeme))))
                        }
                    }
                    TokenType::Less => {
                        match (op1, op2) {
                            (Literal::Number(num1), Literal::Number(num2)) => Ok(Literal::Boolean(num1 < num2)),
                            _ => Err(RuntimeError::TypeError(op.clone(), String::from(format!("Unsupported operand types for '{}'", &op.lexeme))))
                        }
                    }
                    TokenType::LessEqual => {
                        match (op1, op2) {
                            (Literal::Number(num1), Literal::Number(num2)) => Ok(Literal::Boolean(num1 <= num2)),
                            _ => Err(RuntimeError::TypeError(op.clone(), String::from(format!("Unsupported operand types for '{}'", &op.lexeme))))
                        }
                    }
                    _ => panic!("Failure evaluating binary expression")
                }
            },
            Expr::Unary { op, rhs } => {
                let literal = rhs.evaluate(interpreter_state)?;
                match op.token_type {
                    TokenType::Minus => {
                        match literal {
                            Literal::Number(val) => Ok(Literal::Number(-val)),
                            _ => Err(RuntimeError::TypeError(op.clone(), String::from(format!("Unsupported operand type for unary '{}'", &op.lexeme))))
                        }
                    }
                    TokenType::Bang => {
                        match literal {
                            Literal::Boolean(val) => Ok(Literal::Boolean(!val)),
                            _ => Err(RuntimeError::TypeError(op.clone(), String::from(format!("Unsupported operand type for unary '{}'", &op.lexeme))))
                        }
                    }
                    _ => panic!("Failure evaluating unary expression")
                }
            },
            Expr::Grouping { expr } => Ok(expr.evaluate(interpreter_state)?),
            Expr::Literal { literal } => Ok(literal),
            Expr::Variable { name } => {
                match interpreter_state.globals.get(&name.lexeme) {
                    Some(val) => Ok(val.clone()),
                    None => Err(RuntimeError::NameError(name))
                }
            }  
        }
    }
}
