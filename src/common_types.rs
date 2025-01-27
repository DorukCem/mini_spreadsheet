use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    CellName(String),
    Number(f64),
    StringLiteral(String),
    Plus,
    Minus,
    Division,
    Multiply,
    LParen,
    RParen,
    Colon,
    Comma,
    FunctionName(String),
    Bool(bool),

    // logical operators
    Equals,        // ==
    NotEquals,     // !=
    GreaterThan,   // >
    LessThan,      // <
    GreaterEquals, // >=
    LessEquals,    // <=
    And,           // &&
    Or,            // ||
    Not,           // !
}

impl Token {
    #[must_use]
    pub fn get_precedence(&self) -> usize {
        match &self {
            Token::Or => 0,
            Token::And => 1,
            Token::Equals | Token::NotEquals | 
            Token::GreaterThan | Token::LessThan |
            Token::GreaterEquals | Token::LessEquals => 2,
            Token::Plus | Token::Minus => 3,
            Token::Division | Token::Multiply => 4,
            Token::Not => 5,
            _ => 0,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AST {
    CellName(String),
    Value(Value),
    BinaryOp {
        op: Token,
        left: Box<AST>,
        right: Box<AST>,
    },
    UnaryOp {
        op: Token,
        expr: Box<AST>,
    },
    Range {
        from: String,
        to: String,
    },
    FunctionCall {
        name: String,
        arguments: Vec<AST>,
    },
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub ast: AST,
    pub dependencies: Vec<Index>,
}

#[derive(Debug, Clone)]
pub enum ParsedCell {
    Value(Value),
    Expr(Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Text(String),
    Number(f64),
    Bool(bool),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Text(s) => write!(f, "{s}"),
            Value::Number(num) => write!(f, "{num}"),
            Value::Bool(bool) => write!(f, "{}", bool.to_string().to_uppercase()),
        }
    }
}

impl Value {
    #[must_use]
    pub fn add(&self, other: Value) -> Option<Value> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Some(Value::Number(a + b)),
            (Value::Text(a), Value::Text(b)) => Some(Value::Text(a.clone() + &b)),
            _ => None,
        }
    }

    #[must_use]
    pub fn sub(&self, other: Value) -> Option<Value> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Some(Value::Number(a - b)),
            _ => None,
        }
    }

    #[must_use]
    pub fn div(&self, other: Value) -> Option<Value> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Some(Value::Number(a / b)),
            _ => None,
        }
    }

    #[must_use]
    pub fn mult(&self, other: Value) -> Option<Value> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Some(Value::Number(a * b)),
            _ => None,
        }
    }

    pub fn and(&self, other: Value) -> Option<Value> {
        match (self, other) {
            (Value::Bool(a), Value::Bool(b)) => Some(Value::Bool(*a && b)),
            _ => None,
        }
    }
    pub fn or(&self, other: Value) -> Option<Value> {
        match (self, other) {
            (Value::Bool(a), Value::Bool(b)) => Some(Value::Bool(*a || b)),
            _ => None,
        }
    }

    pub fn greater_than(&self, other: Value) -> Option<Value> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Some(Value::Bool(a > &b)),
            _ => None,
        }
    }
    pub fn less_than(&self, other: Value) -> Option<Value> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Some(Value::Bool(a < &b)),
            _ => None,
        }
    }

    pub fn greater_equals(&self, other: Value) -> Option<Value> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Some(Value::Bool(a >= &b)),
            _ => None,
        }
    }

    pub fn less_equals(&self, other: Value) -> Option<Value> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Some(Value::Bool(a <= &b)),
            _ => None,
        }
    }

}

#[derive(Debug, Clone)]
pub struct ParseError(pub String);

#[derive(Debug, Clone)]
pub enum ComputeError {
    ParseError(String),
    TypeError(String),
    UnfindableReference(String),
    Cycle,
    UnknownFunction(String),
    InvalidArgument(String)
}

impl Display for ComputeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComputeError::ParseError(_) => write!(f, "!-PARSE ERROR-!"),
            ComputeError::TypeError(_) => write!(f, "!-TYPE ERROR-!"),
            ComputeError::UnfindableReference(_) => write!(f, "!-REFERENCE ERROR-!"),
            ComputeError::Cycle => write!(f, "!-CYCLIC REFERENCE-!"),
            ComputeError::UnknownFunction(_) => write!(f, "!-UNKNOWN FUNCTION-!"),
            ComputeError::InvalidArgument(_) => write!(f, "!-INVALID FUNCTION ARGUMENT-!"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cell {
    pub needs_compute: bool,
    pub raw_representation: String,
    pub parsed_representation: Option<Result<ParsedCell, ParseError>>,
    pub computed_value: Option<Result<Value, ComputeError>>,
}

impl Cell {
    #[must_use]
    pub fn from_raw(raw: String) -> Self {
        Self {
            raw_representation: raw,
            parsed_representation: None,
            computed_value: None,
            needs_compute: true,
        }
    }
}

#[derive(PartialEq, Hash, Eq, Debug, Clone, Copy, Ord, PartialOrd)]
pub struct Index {
    pub x: usize,
    pub y: usize,
}

