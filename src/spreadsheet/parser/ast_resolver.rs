use builtin_functions::get_func;

use crate::common_types::{ComputeError, Index, Token, Value, AST};
mod builtin_functions;
pub trait VarContext {
    fn get_variable(&self, index: Index) -> Option<Result<Value, ComputeError>>;
}

pub struct ASTResolver {}

impl ASTResolver {
    pub fn resolve(ast: &AST, variables: &dyn VarContext) -> Result<Value, ComputeError> {
        match ast {
            AST::Value(value) => Ok(value.clone()),
            AST::CellName(name) => match variables.get_variable(
                Self::get_cell_idx(name)
                    .ok_or(ComputeError::ParseError("Invalid cell name".to_string()))?,
            ) {
                Some(value) => value,
                None => Err(ComputeError::UnfindableReference(format!(
                    "Could not find variable {name} with in context"
                ))),
            },
            AST::BinaryOp { op, left, right } => {
                let left_resolved = Self::resolve(left, variables)?;
                let right_resolved = Self::resolve(right, variables)?;

                match op {
                    Token::Plus => {
                        left_resolved
                            .add(right_resolved)
                            .ok_or(ComputeError::TypeError(
                                "Addition requires two numeric values".to_string(),
                            ))
                    }
                    Token::Minus => {
                        left_resolved
                            .sub(right_resolved)
                            .ok_or(ComputeError::TypeError(
                                "Subtraction requires two numeric values".to_string(),
                            ))
                    }
                    Token::Division => {
                        left_resolved
                            .div(right_resolved)
                            .ok_or(ComputeError::TypeError(
                                "Division requires two numeric values".to_string(),
                            ))
                    }
                    Token::Multiply => {
                        left_resolved
                            .mult(right_resolved)
                            .ok_or(ComputeError::TypeError(
                                "Multiplication requires two numeric values".to_string(),
                            ))
                    }

                    Token::Equals => Ok(Value::Bool(left_resolved.eq(&right_resolved))),
                    Token::NotEquals => Ok(Value::Bool(left_resolved.ne(&right_resolved))),
                    Token::GreaterThan => {
                        left_resolved
                            .greater_than(right_resolved)
                            .ok_or(ComputeError::TypeError(
                                "Greater than comparison requires two numeric values".to_string(),
                            ))
                    }
                    Token::LessThan => {
                        left_resolved
                            .less_than(right_resolved)
                            .ok_or(ComputeError::TypeError(
                                "Less than comparison requires two numeric values".to_string(),
                            ))
                    }
                    Token::GreaterEquals => {
                        left_resolved
                            .greater_equals(right_resolved)
                            .ok_or(ComputeError::TypeError(
                                "Greater or equal comparison requires two numeric values"
                                    .to_string(),
                            ))
                    }
                    Token::LessEquals => {
                        left_resolved
                            .less_equals(right_resolved)
                            .ok_or(ComputeError::TypeError(
                                "Less or equal comparison requires two numeric values".to_string(),
                            ))
                    }
                    Token::And => left_resolved
                        .and(right_resolved)
                        .ok_or(ComputeError::TypeError(
                            "Logical AND requires two boolean values".to_string(),
                        )),
                    Token::Or => left_resolved
                        .or(right_resolved)
                        .ok_or(ComputeError::TypeError(
                            "Logical OR requires two boolean values".to_string(),
                        )),
                    other => panic!("{other:?} is not a binary operator"),
                }
            }
            AST::Range { from: _, to: _ } => Err(ComputeError::TypeError(
                "Ranges can only appear as function arguments".to_owned(),
            )),

            AST::FunctionCall { name, arguments } => {
                let mut resolved_args = Vec::new();
                for arg in arguments {
                    match arg {
                        AST::Range { from, to } => {
                            for index in Self::range_to_indeces(from, to) {
                                if let Some(var) = variables.get_variable(index) {
                                    resolved_args.push(var?)
                                }
                            }
                        }
                        ast => resolved_args.push(Self::resolve(ast, variables)?),
                    }
                }

                if let Some(func) = get_func(name) {
                    func(resolved_args)
                } else {
                    Err(ComputeError::UnknownFunction(name.to_owned()))
                }
            }
            AST::UnaryOp { op, expr } => {
                matches!(op, Token::Not);
                if let Value::Bool(boolean) = Self::resolve(expr, variables)? {
                    Ok(Value::Bool(!boolean))
                } else {
                    Err(ComputeError::TypeError(
                        "Not(!) operator can only work on boolean expressions".to_owned(),
                    ))
                }
            }
        }
    }

    pub fn get_cell_idx(cell_name: &str) -> Option<Index> {
        let mut x: usize = 0;
        let mut y = 0;

        for (i, c) in cell_name.chars().enumerate() {
            if c.is_ascii_digit() {
                // Parse row number
                y = cell_name[i..].parse::<usize>().ok()?;
                break;
            } else {
                // Parse column letters
                x = x * 26 + (c as usize - 'A' as usize + 1);
            }
        }
        if x == 0 || y == 0 {
            return None;
        }
        // Adjust for 0-based indexing
        Some(Index { x: x - 1, y: y - 1 })
    }

    fn range_to_indeces(from: &str, to: &str) -> Vec<Index> {
        let start = Self::get_cell_idx(from).unwrap_or(Index{ x:0, y:0 });
        let end = Self::get_cell_idx(to).unwrap_or(Index{ x:0, y:0 });
        let mut indices = Vec::new();
        for x in start.x..=end.x {
            for y in start.y..=end.y {
                indices.push(Index { x, y });
            }
        }

        indices
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockVarContext {
        variables: HashMap<Index, Value>,
    }

    impl VarContext for MockVarContext {
        fn get_variable(&self, index: Index) -> Option<Result<Value, ComputeError>> {
            self.variables.get(&index).cloned().map(Ok)
        }
    }

    impl MockVarContext {
        fn new(variables: HashMap<Index, Value>) -> Self {
            Self { variables }
        }
    }

    #[test]
    fn test_resolve_value_ast() {
        let variables = MockVarContext::new(HashMap::new());
        let ast = AST::Value(Value::Number(42.0));

        let result = ASTResolver::resolve(&ast, &variables).unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_resolve_cellname_ast() {
        let mut vars = HashMap::new();
        vars.insert(Index { x: 0, y: 0 }, Value::Number(10.0));

        let variables = MockVarContext::new(vars);
        let ast = AST::CellName("A1".to_string());

        let result = ASTResolver::resolve(&ast, &variables).unwrap();
        assert_eq!(result, Value::Number(10.0));
    }

    #[test]
    fn test_resolve_binary_op_addition() {
        let mut vars = HashMap::new();
        vars.insert(Index { x: 0, y: 0 }, Value::Number(10.0));
        vars.insert(Index { x: 1, y: 0 }, Value::Number(20.0));

        let variables = MockVarContext::new(vars);
        let ast = AST::BinaryOp {
            op: Token::Plus,
            left: Box::new(AST::CellName("A1".to_string())),
            right: Box::new(AST::CellName("B1".to_string())),
        };

        let result = ASTResolver::resolve(&ast, &variables).unwrap();
        assert_eq!(result, Value::Number(30.0));
    }

    #[test]
    fn test_resolve_binary_op_subtraction() {
        let mut vars = HashMap::new();
        vars.insert(Index { x: 0, y: 0 }, Value::Number(30.0));
        vars.insert(Index { x: 1, y: 0 }, Value::Number(20.0));

        let variables = MockVarContext::new(vars);
        let ast = AST::BinaryOp {
            op: Token::Minus,
            left: Box::new(AST::CellName("A1".to_string())),
            right: Box::new(AST::CellName("B1".to_string())),
        };

        let result = ASTResolver::resolve(&ast, &variables).unwrap();
        assert_eq!(result, Value::Number(10.0));
    }

    #[test]
    fn test_resolve_binary_op_multiplication() {
        let mut vars = HashMap::new();
        vars.insert(Index { x: 0, y: 0 }, Value::Number(3.0));
        vars.insert(Index { x: 1, y: 0 }, Value::Number(4.0));

        let variables = MockVarContext::new(vars);
        let ast = AST::BinaryOp {
            op: Token::Multiply,
            left: Box::new(AST::CellName("A1".to_string())),
            right: Box::new(AST::CellName("B1".to_string())),
        };

        let result = ASTResolver::resolve(&ast, &variables).unwrap();
        assert_eq!(result, Value::Number(12.0));
    }

    #[test]
    fn test_resolve_binary_op_division() {
        let mut vars = HashMap::new();
        vars.insert(Index { x: 0, y: 0 }, Value::Number(20.0));
        vars.insert(Index { x: 1, y: 0 }, Value::Number(4.0));

        let variables = MockVarContext::new(vars);
        let ast = AST::BinaryOp {
            op: Token::Division,
            left: Box::new(AST::CellName("A1".to_string())),
            right: Box::new(AST::CellName("B1".to_string())),
        };

        let result = ASTResolver::resolve(&ast, &variables).unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    #[should_panic]
    fn test_resolve_missing_cellname() {
        let variables = MockVarContext::new(HashMap::new());
        let ast = AST::CellName("A1".to_string());

        // This should panic because "A1" is not in the context
        ASTResolver::resolve(&ast, &variables).unwrap();
    }

    #[test]
    fn test_resolve_deep_tree_addition_multiplication() {
        let mut vars = HashMap::new();
        vars.insert(Index { x: 0, y: 0 }, Value::Number(2.0));
        vars.insert(Index { x: 1, y: 0 }, Value::Number(3.0));
        vars.insert(Index { x: 2, y: 0 }, Value::Number(4.0));

        let variables = MockVarContext::new(vars);
        let ast = AST::BinaryOp {
            op: Token::Plus,
            left: Box::new(AST::BinaryOp {
                op: Token::Multiply,
                left: Box::new(AST::CellName("A1".to_string())),
                right: Box::new(AST::CellName("B1".to_string())),
            }),
            right: Box::new(AST::CellName("C1".to_string())),
        };

        let result = ASTResolver::resolve(&ast, &variables).unwrap();
        assert_eq!(result, Value::Number(10.0));
    }

    #[test]
    fn test_resolve_deep_tree_subtraction_division() {
        let mut vars = HashMap::new();
        vars.insert(Index { x: 0, y: 0 }, Value::Number(20.0));
        vars.insert(Index { x: 1, y: 0 }, Value::Number(4.0));
        vars.insert(Index { x: 2, y: 0 }, Value::Number(2.0));

        let variables = MockVarContext::new(vars);
        let ast = AST::BinaryOp {
            op: Token::Minus,
            left: Box::new(AST::BinaryOp {
                op: Token::Division,
                left: Box::new(AST::CellName("A1".to_string())),
                right: Box::new(AST::CellName("B1".to_string())),
            }),
            right: Box::new(AST::CellName("C1".to_string())),
        };

        let result = ASTResolver::resolve(&ast, &variables).unwrap();
        assert_eq!(result, Value::Number(3.0));
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::common_types::{Index, Token};
        use std::collections::HashMap;

        #[test]
        fn test_simple_sum() {
            let mut vars = HashMap::new();
            vars.insert(Index { x: 0, y: 0 }, Value::Number(10.0));
            vars.insert(Index { x: 1, y: 0 }, Value::Number(20.0));
            let variables = MockVarContext::new(vars);

            let ast = AST::FunctionCall {
                name: "sum".to_string(),
                arguments: vec![
                    AST::CellName("A1".to_string()),
                    AST::CellName("B1".to_string()),
                ],
            };

            let result = ASTResolver::resolve(&ast, &variables).unwrap();
            assert_eq!(result, Value::Number(30.0));
        }

        #[test]
        fn test_sum_with_range() {
            let mut vars = HashMap::new();
            vars.insert(Index { x: 0, y: 0 }, Value::Number(10.0));
            vars.insert(Index { x: 0, y: 1 }, Value::Number(20.0));
            vars.insert(Index { x: 0, y: 2 }, Value::Number(30.0));
            let variables = MockVarContext::new(vars);

            let ast = AST::FunctionCall {
                name: "sum".to_string(),
                arguments: vec![AST::Range {
                    from: "A1".to_string(),
                    to: "A3".to_string(),
                }],
            };

            let result = ASTResolver::resolve(&ast, &variables).unwrap();
            assert_eq!(result, Value::Number(60.0));
        }

        #[test]
        fn test_sum_with_mixed_arguments() {
            let mut vars = HashMap::new();
            vars.insert(Index { x: 0, y: 0 }, Value::Number(10.0));
            vars.insert(Index { x: 0, y: 1 }, Value::Number(20.0));
            let variables = MockVarContext::new(vars);

            let ast = AST::FunctionCall {
                name: "sum".to_string(),
                arguments: vec![
                    AST::Range {
                        from: "A1".to_string(),
                        to: "A2".to_string(),
                    },
                    AST::Value(Value::Number(5.0)),
                ],
            };

            let result = ASTResolver::resolve(&ast, &variables).unwrap();
            assert_eq!(result, Value::Number(35.0));
        }

        #[test]
        fn test_sum_with_expression() {
            let mut vars = HashMap::new();
            vars.insert(Index { x: 0, y: 0 }, Value::Number(10.0));
            let variables = MockVarContext::new(vars);

            let ast = AST::FunctionCall {
                name: "sum".to_string(),
                arguments: vec![
                    AST::BinaryOp {
                        op: Token::Plus,
                        left: Box::new(AST::CellName("A1".to_string())),
                        right: Box::new(AST::Value(Value::Number(5.0))),
                    },
                    AST::Value(Value::Number(15.0)),
                ],
            };

            let result = ASTResolver::resolve(&ast, &variables).unwrap();
            assert_eq!(result, Value::Number(30.0));
        }

        #[test]
        fn test_sum_with_nested_function() {
            let mut vars = HashMap::new();
            vars.insert(Index { x: 0, y: 0 }, Value::Number(10.0));
            vars.insert(Index { x: 1, y: 0 }, Value::Number(20.0));
            let variables = MockVarContext::new(vars);

            let ast = AST::FunctionCall {
                name: "sum".to_string(),
                arguments: vec![
                    AST::FunctionCall {
                        name: "sum".to_string(),
                        arguments: vec![
                            AST::CellName("A1".to_string()),
                            AST::CellName("B1".to_string()),
                        ],
                    },
                    AST::Value(Value::Number(5.0)),
                ],
            };

            let result = ASTResolver::resolve(&ast, &variables).unwrap();
            assert_eq!(result, Value::Number(35.0));
        }

        #[test]
        fn test_unknown_function() {
            let variables = MockVarContext::new(HashMap::new());

            let ast = AST::FunctionCall {
                name: "nonexistent".to_string(),
                arguments: vec![AST::Value(Value::Number(10.0))],
            };

            let result = ASTResolver::resolve(&ast, &variables);
            assert!(matches!(result, Err(ComputeError::UnknownFunction(_))));
        }

        #[test]
        fn test_sum_invalid_arg_error() {
            let mut vars = HashMap::new();
            vars.insert(Index { x: 0, y: 0 }, Value::Text("a".to_string()));
            let variables = MockVarContext::new(vars);

            let ast = AST::FunctionCall {
                name: "sum".to_string(),
                arguments: vec![AST::CellName("A1".to_string())],
            };

            let result = ASTResolver::resolve(&ast, &variables);
            assert!(matches!(result, Err(ComputeError::InvalidArgument(_))));
        }

        #[test]
        fn test_sum_empty_range() {
            let variables = MockVarContext::new(HashMap::new());

            let ast = AST::FunctionCall {
                name: "sum".to_string(),
                arguments: vec![AST::Range {
                    from: "A1".to_string(),
                    to: "A2".to_string(),
                }],
            };

            let result = ASTResolver::resolve(&ast, &variables).unwrap();
            assert_eq!(result, Value::Number(0.0)); // Sum of empty range should be 0
        }
    }

    #[test]
    fn test_simple_boolean_value() {
        let variables = MockVarContext::new(HashMap::new());
        let ast = AST::Value(Value::Bool(true));
        let result = ASTResolver::resolve(&ast, &variables).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_logical_not() {
        let variables = MockVarContext::new(HashMap::new());
        let ast = AST::UnaryOp {
            op: Token::Not,
            expr: Box::new(AST::Value(Value::Bool(true))),
        };
        let result = ASTResolver::resolve(&ast, &variables).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_logical_and() {
        let mut vars = HashMap::new();
        vars.insert(Index { x: 0, y: 0 }, Value::Bool(true));
        vars.insert(Index { x: 0, y: 1 }, Value::Bool(false));
        let variables = MockVarContext::new(vars);

        // Test true && false
        let ast = AST::BinaryOp {
            op: Token::And,
            left: Box::new(AST::CellName("A1".to_string())),
            right: Box::new(AST::CellName("A2".to_string())),
        };
        let result = ASTResolver::resolve(&ast, &variables).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_logical_or() {
        let mut vars = HashMap::new();
        vars.insert(Index { x: 0, y: 0 }, Value::Bool(true));
        vars.insert(Index { x: 0, y: 1 }, Value::Bool(false));
        let variables = MockVarContext::new(vars);

        // Test true || false
        let ast = AST::BinaryOp {
            op: Token::Or,
            left: Box::new(AST::CellName("A1".to_string())),
            right: Box::new(AST::CellName("A2".to_string())),
        };
        let result = ASTResolver::resolve(&ast, &variables).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_comparison_operators() {
        let mut vars = HashMap::new();
        vars.insert(Index { x: 0, y: 0 }, Value::Number(10.0));
        vars.insert(Index { x: 0, y: 1 }, Value::Number(20.0));
        let variables = MockVarContext::new(vars);

        // Test greater than
        let ast = AST::BinaryOp {
            op: Token::GreaterThan,
            left: Box::new(AST::CellName("A1".to_string())),
            right: Box::new(AST::CellName("A2".to_string())),
        };
        let result = ASTResolver::resolve(&ast, &variables).unwrap();
        assert_eq!(result, Value::Bool(false));

        // Test less than
        let ast = AST::BinaryOp {
            op: Token::LessThan,
            left: Box::new(AST::CellName("A1".to_string())),
            right: Box::new(AST::CellName("A2".to_string())),
        };
        let result = ASTResolver::resolve(&ast, &variables).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_equality_comparison() {
        let mut vars = HashMap::new();
        vars.insert(Index { x: 0, y: 0 }, Value::Number(10.0));
        vars.insert(Index { x: 0, y: 1 }, Value::Number(10.0));
        vars.insert(Index { x: 0, y: 2 }, Value::Text("hello".to_string()));
        let variables = MockVarContext::new(vars);

        // Test number equality
        let ast = AST::BinaryOp {
            op: Token::Equals,
            left: Box::new(AST::CellName("A1".to_string())),
            right: Box::new(AST::CellName("A2".to_string())),
        };
        let result = ASTResolver::resolve(&ast, &variables).unwrap();
        assert_eq!(result, Value::Bool(true));

        // Test different types equality
        let ast = AST::BinaryOp {
            op: Token::Equals,
            left: Box::new(AST::CellName("A1".to_string())),
            right: Box::new(AST::CellName("A3".to_string())),
        };
        let result = ASTResolver::resolve(&ast, &variables).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_complex_logical_expression() {
        let mut vars = HashMap::new();
        vars.insert(Index { x: 0, y: 0 }, Value::Bool(true));
        vars.insert(Index { x: 0, y: 1 }, Value::Number(15.0));
        vars.insert(Index { x: 0, y: 2 }, Value::Number(10.0));
        let variables = MockVarContext::new(vars);

        // Test (A1 && (A2 > A3))
        let ast = AST::BinaryOp {
            op: Token::And,
            left: Box::new(AST::CellName("A1".to_string())),
            right: Box::new(AST::BinaryOp {
                op: Token::GreaterThan,
                left: Box::new(AST::CellName("A2".to_string())),
                right: Box::new(AST::CellName("A3".to_string())),
            }),
        };
        let result = ASTResolver::resolve(&ast, &variables).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_type_error_in_logical_operation() {
        let mut vars = HashMap::new();
        vars.insert(Index { x: 0, y: 0 }, Value::Bool(true));
        vars.insert(
            Index { x: 0, y: 1 },
            Value::Text("not a boolean".to_string()),
        );
        let variables = MockVarContext::new(vars);

        let ast = AST::BinaryOp {
            op: Token::And,
            left: Box::new(AST::CellName("A1".to_string())),
            right: Box::new(AST::CellName("A2".to_string())),
        };
        let result = ASTResolver::resolve(&ast, &variables);
        assert!(matches!(result, Err(ComputeError::TypeError(_))));
    }

    #[test]
    fn test_not_with_non_boolean() {
        let mut vars = HashMap::new();
        vars.insert(Index { x: 0, y: 0 }, Value::Number(42.0));
        let variables = MockVarContext::new(vars);

        let ast = AST::UnaryOp {
            op: Token::Not,
            expr: Box::new(AST::CellName("A1".to_string())),
        };
        let result = ASTResolver::resolve(&ast, &variables);
        assert!(matches!(result, Err(ComputeError::TypeError(_))));
    }

    #[test]
    fn test_comparison_type_mismatch() {
        let mut vars = HashMap::new();
        vars.insert(Index { x: 0, y: 0 }, Value::Number(42.0));
        vars.insert(Index { x: 0, y: 1 }, Value::Bool(true));
        let variables = MockVarContext::new(vars);

        let ast = AST::BinaryOp {
            op: Token::GreaterThan,
            left: Box::new(AST::CellName("A1".to_string())),
            right: Box::new(AST::CellName("A2".to_string())),
        };
        let result = ASTResolver::resolve(&ast, &variables);
        assert!(matches!(result, Err(ComputeError::TypeError(_))));
    }
}
