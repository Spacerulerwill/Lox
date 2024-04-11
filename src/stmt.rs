use std::collections::HashMap;

use crate::expr::{Expr, RuntimeError};
use crate::tokenizer::{Token, Literal};
use crate::InterpreterState;

#[derive(Debug)]
pub enum Stmt {
    Expression {
        expr: Expr
    },
    Print {
        expr: Expr
    },
    Var {
        token: Token,
        intializer: Expr
    },
}

impl Stmt {
    pub fn interpret(self, interpreter_state: &mut InterpreterState) -> Result<(), RuntimeError> {
        match self {
            Stmt::Expression { expr } => Ok({
                expr.evaluate(interpreter_state)?;
            }),
            Stmt::Print { expr } => {
                Ok(println!("{}", expr.evaluate(interpreter_state)?))
            },
            Stmt::Var { token, intializer } => {
                let literal = intializer.evaluate(interpreter_state)?;
                interpreter_state.globals.insert(token.lexeme, literal);
                Ok(())
            }
        }
    }
}