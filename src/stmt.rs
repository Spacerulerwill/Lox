use std::collections::HashMap;

use crate::expr::{Expr, RuntimeError};
use crate::tokenizer::Literal;

#[derive(Debug)]
pub enum Stmt {
    Expression {
        expr: Expr
    },
    Print {
        expr: Expr
    },
}

impl Stmt {
    pub fn interpret(self, values: &mut HashMap<String, Literal>) -> Result<(), RuntimeError> {
        match self {
            Stmt::Expression { expr } => Ok({
                expr.evaluate()?;
            }),
            Stmt::Print { expr } => {
                Ok(println!("{}", expr.evaluate()?))
            },
        }
    }
}