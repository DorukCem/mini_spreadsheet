use super::ast::Token;

pub struct ExpressionTokenizer {
    index: usize,
    chars: Vec<char>,
}

#[derive(Debug)]
pub enum TokenizeError {
    UnexpectedCharacter(char),
    InvalidCellName(String),
}

impl ExpressionTokenizer {
    pub fn new(chars: Vec<char>) -> Self {
        Self { index: 0, chars }
    }

    pub fn tokenize_expression(&mut self) -> Result<Vec<Token>, TokenizeError> {
        self.skip_whitespace();
        let mut expr_tokens = Vec::new();
        while !self.is_done() {
            let token = match self.peek().expect("Should never fail") {
                '+' | '-' | '/' | '*' | '(' | ')' => self.parse_operator(),
                letter if letter.is_uppercase() => self.parse_cell_name().unwrap(),
                unknown => return Err(TokenizeError::UnexpectedCharacter(*unknown)),
            };

            expr_tokens.push(token);

            self.skip_whitespace();
        }

        Ok(expr_tokens)
    }

    fn parse_cell_name(&mut self) -> Result<Token, TokenizeError> {
        // [A-Z]+\d+

        let mut is_valid = false;
        let mut cell_name = String::new();

        // Collect the uppercase letters
        while let Some(&ch) = self.peek() {
            if ch.is_ascii_uppercase() {
                cell_name.push(ch);
                self.pop();
            } else {
                break;
            }
        }

        // Ensure there are letters
        if cell_name.is_empty() {
            return Err(TokenizeError::InvalidCellName("".to_string()));
        }

        // Collect the digits
        while let Some(&ch) = self.peek() {
            if ch.is_ascii_digit() {
                cell_name.push(ch);
                self.pop();
                is_valid = true;
            } else {
                break;
            }
        }

        // Ensure the format was valid ``
        if !is_valid {
            return Err(TokenizeError::InvalidCellName(cell_name));
        }

        Ok(Token::CellName(cell_name))
    }

    fn parse_operator(&mut self) -> Token {
        match self.pop().expect("Shoud never fail") {
            '+' => Token::Plus,
            '-' => Token::Minus,
            '/' => Token::Division,
            '*' => Token::Multiply,
            '(' => Token::LParen,
            ')' => Token::RParen,
            _ => unreachable!(),
        }
    }

    fn peek(&self) -> Option<&char> {
        self.chars.get(self.index)
    }

    fn is_done(&self) -> bool {
        self.index >= self.chars.len()
    }

    fn pop(&mut self) -> Option<&char> {
        let val = self.chars.get(self.index);
        self.index += 1;
        val
    }

    fn skip_whitespace(&mut self) -> bool {
        while !self.is_done() && (self.peek() == Some(&' ') || self.peek() == Some(&'\n')) {
            self.pop();
        }
        // Some error occured
        if self.is_done() {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_expression() {
        let s = "A1 + A2";
        let tokens = ExpressionTokenizer::new(s.chars().collect()).tokenize_expression().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::CellName("A1".to_string()),
                Token::Plus,
                Token::CellName("A2".to_string())
            ]
        );
    }

    #[test]
    fn test_expression_with_parentheses() {
        let s = "(A1 + B2) * C3";
        let tokens = ExpressionTokenizer::new(s.chars().collect()).tokenize_expression().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::LParen,
                Token::CellName("A1".to_string()),
                Token::Plus,
                Token::CellName("B2".to_string()),
                Token::RParen,
                Token::Multiply,
                Token::CellName("C3".to_string())
            ]
        );
    }

    #[test]
    fn test_expression_with_division_and_whitespace() {
        let s = "  A1   /   B2 ";
        let tokens = ExpressionTokenizer::new(s.chars().collect()).tokenize_expression().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::CellName("A1".to_string()),
                Token::Division,
                Token::CellName("B2".to_string())
            ]
        );
    }

    #[test]
    fn test_complex_expression() {
        let s = "((A1 + B2) - C3) * D4 / E5";
        let tokens = ExpressionTokenizer::new(s.chars().collect()).tokenize_expression().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::LParen,
                Token::LParen,
                Token::CellName("A1".to_string()),
                Token::Plus,
                Token::CellName("B2".to_string()),
                Token::RParen,
                Token::Minus,
                Token::CellName("C3".to_string()),
                Token::RParen,
                Token::Multiply,
                Token::CellName("D4".to_string()),
                Token::Division,
                Token::CellName("E5".to_string())
            ]
        );
    }

    #[test]
    fn test_empty_expression() {
        let s = "";
        let tokens = ExpressionTokenizer::new(s.chars().collect()).tokenize_expression().unwrap();
        assert!(
            tokens.is_empty(),
            "Expected empty token list for empty expression"
        );
    }

    #[test]
    fn test_expression_with_extra_whitespace() {
        let s = "   A1    +     A2   ";
        let tokens = ExpressionTokenizer::new(s.chars().collect()).tokenize_expression().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::CellName("A1".to_string()),
                Token::Plus,
                Token::CellName("A2".to_string())
            ]
        );
    }

}
