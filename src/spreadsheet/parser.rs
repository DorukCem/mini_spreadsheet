use ast_creator::{ASTCreateError, ASTCreator};
use tokenizer::ExpressionTokenizer;

use crate::{common_functions::get_cell_idx, common_types::{ParseError, Token, Value}};

use super::{Cell, Expression, Index, ParsedCell};

pub mod ast_creator;
pub mod ast_resolver;
pub mod dependancy_graph;
pub mod tokenizer;

pub struct CellParser {}

impl CellParser {
    pub fn parse_cell(cell: &mut Cell) {
        let raw_cell = &cell.raw_representation;
        if raw_cell.is_empty() {
            panic!("Parsing empty cell")
        }

        let parsed_cell = match raw_cell.chars().nth(0).expect("Should never fail") {
            '=' => Self::parse_expression(raw_cell),
            d if d.is_ascii_digit() || d == '-' || d == '+' => match raw_cell.parse() {
                Ok(number) => Ok(ParsedCell::Value(Value::Number(number))),
                Err(e) => Err(ParseError(format!(
                    "Had error: -{e}- parsing number {raw_cell}"
                ))),
            },
            _ => {
                let s = raw_cell.to_string();
                if s == "TRUE" {
                    Ok(ParsedCell::Value(Value::Bool(true)))
                } else if s == "FALSE" {
                    Ok(ParsedCell::Value(Value::Bool(false)))
                } else {
                    Ok(ParsedCell::Value(Value::Text(s)))
                }
            }
        };

        cell.parsed_representation = Some(parsed_cell);
    }

    fn parse_expression(s: &str) -> Result<ParsedCell, ParseError> {
        let tokens = ExpressionTokenizer::new(s[1..].chars().collect())
            .tokenize_expression()
            .map_err(|e| match e {
                tokenizer::TokenizeError::UnexpectedCharacter(c) => {
                    ParseError(format!("Unexpected characther: {c}"))
                }
                tokenizer::TokenizeError::InvalidCellName(name) => {
                    ParseError(format!("Invalid cell name: {name}"))
                }
                tokenizer::TokenizeError::InvalidNumber(num) => {
                    ParseError(format!("Invalid number format: {num}"))
                }
                tokenizer::TokenizeError::UnterminatedString => {
                    ParseError("Unterminated String".to_string())
                }
            })?;

        let dependencies = Self::find_dependants(&tokens);
        let ast = ASTCreator::new(tokens.into_iter())
            .parse()
            .map_err(|e| match e {
                ASTCreateError::UnexpectedToken => ParseError("Unexpected Token".to_string()),
                ASTCreateError::MismatchedParentheses => {
                    ParseError("Mismatched Parentheses".to_string())
                }
                ASTCreateError::InvalidRange => ParseError("Invalid Range Expression".to_string()),
            })?;
        let expr = Expression { ast, dependencies };
        Ok(ParsedCell::Expr(expr))
    }

    fn find_dependants(tokens: &[Token]) -> Vec<Index> {
        let cells = tokens
            .iter()
            .filter_map(|x| match x {
                Token::CellName(name) => get_cell_idx(name),
                _ => None,
            })
            .collect();

        cells
    }
}
