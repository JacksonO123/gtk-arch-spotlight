use std::iter::Peekable;

#[derive(Debug)]
pub enum ParseError {
    InvalidNumber,
    UnexpectedCharacter,
    UnexpectedToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Operator {
    Exponent,
    Mult,
    Div,
    Sub,
    Add,
}

#[derive(Debug, Clone, PartialEq)]
enum Node {
    Number(f64),
    Expr(Box<Node>, Operator, Box<Node>),
    Group(Box<Node>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ParenType {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Token<'a> {
    Number(&'a str),
    Operator(Operator),
    Paren(ParenType),
}

fn parse_str(expr_str: &str) -> Result<Option<Node>, ParseError> {
    let tokens = tokenize(expr_str)?;
    let mut peek_iter = tokens.into_iter().peekable();
    let mut res = parse_expr(&mut peek_iter)?;

    while peek_iter.peek().is_some() {
        res = parse_from_unit_node(&mut peek_iter, res, true)?;
    }

    Ok(res)
}

fn parse_expr<'a, T>(tokens: &mut Peekable<T>) -> Result<Option<Node>, ParseError>
where
    T: Iterator<Item = Token<'a>> + Clone,
{
    let Some(first) = tokens.next() else {
        return Ok(None);
    };

    let unit_node = match first {
        Token::Number(num_str) => {
            let lhs_num: f64 = num_str.parse().expect("not parsable to f64");
            Some(Node::Number(lhs_num))
        }
        Token::Paren(ParenType::Left) => {
            let res = parse_expr(tokens)?;
            if let Some(found_token) = tokens.next()
                && found_token != Token::Paren(ParenType::Right)
            {
                return Err(ParseError::UnexpectedToken);
            } else {
                res.map(|node| Node::Group(Box::new(node)))
            }
        }
        _ => {
            return Err(ParseError::UnexpectedToken);
        }
    };

    parse_from_unit_node(tokens, unit_node, false)
}

fn parse_from_unit_node<'a, T>(
    tokens: &mut Peekable<T>,
    node: Option<Node>,
    top: bool,
) -> Result<Option<Node>, ParseError>
where
    T: Iterator<Item = Token<'a>> + Clone,
{
    let Some(next) = tokens.peek() else {
        return Ok(node);
    };

    let operator = match next {
        Token::Operator(operator) => *operator,
        Token::Paren(ParenType::Right) => {
            if top {
                tokens.next();
            }

            return Ok(node);
        }
        _ => {
            return Err(ParseError::UnexpectedToken);
        }
    };

    let Some(unit_node) = node else {
        return Ok(None);
    };

    tokens.next();

    let Some(rhs) = parse_expr(tokens)? else {
        return Ok(None);
    };

    let result_expr = Node::Expr(Box::new(unit_node), operator, Box::new(rhs));
    let result_expr = normalize(result_expr);
    Ok(Some(result_expr))
}

fn tokenize<'a>(expr_str: &'a str) -> Result<Vec<Token<'a>>, ParseError> {
    let mut tokens = vec![];
    let chars = expr_str.chars();

    let mut iter = chars.enumerate().peekable();
    while let Some((i, mut char)) = iter.next() {
        let token = match char {
            ' ' => {
                continue;
            }
            '+' => Token::Operator(Operator::Add),
            '-' => Token::Operator(Operator::Sub),
            '*' => Token::Operator(Operator::Mult),
            '/' => Token::Operator(Operator::Div),
            '^' => Token::Operator(Operator::Exponent),
            '(' => Token::Paren(ParenType::Left),
            ')' => Token::Paren(ParenType::Right),
            _ => {
                if char.is_numeric() || char == '.' {
                    let start = i;
                    let mut end = i;
                    let mut found_period = false;

                    loop {
                        if char == '.' {
                            if found_period {
                                return Err(ParseError::InvalidNumber);
                            }

                            found_period = true;
                        }

                        end += 1;

                        let Some((_, num_char)) = iter.peek() else {
                            break;
                        };

                        if num_char.is_numeric() || (*num_char == '.' && !found_period) {
                            char = *num_char;
                            iter.next();
                        } else {
                            break;
                        }
                    }

                    let slice = &expr_str[start..end];
                    Token::Number(slice)
                } else {
                    return Err(ParseError::UnexpectedCharacter);
                }
            }
        };

        tokens.push(token);
    }

    Ok(tokens)
}

fn normalize(node: Node) -> Node {
    match node {
        Node::Number(_) => node,
        Node::Group(inner) => normalize(*inner),
        Node::Expr(left, operator, right) => {
            let left = normalize(*left);
            let right = normalize(*right);
            rotate_precedence(left, operator, right)
        }
    }
}

fn rotate_precedence(left: Node, operator: Operator, right: Node) -> Node {
    match right {
        Node::Expr(right_left, right_operator, right_right) if operator <= right_operator => {
            let new_left = rotate_precedence(left, operator, *right_left);
            Node::Expr(Box::new(new_left), right_operator, right_right)
        }
        _ => Node::Expr(Box::new(left), operator, Box::new(right)),
    }
}

fn evaluate_node(node: Node) -> f64 {
    match node {
        Node::Number(num) => num,
        Node::Expr(left, operator, right) => {
            let left = evaluate_node(*left);
            let right = evaluate_node(*right);
            match operator {
                Operator::Add => left + right,
                Operator::Sub => left - right,
                Operator::Mult => left * right,
                Operator::Div => left / right,
                Operator::Exponent => left.powf(right),
            }
        }
        Node::Group(inner) => evaluate_node(*inner),
    }
}

pub fn evaluate_str(expr_str: &str) -> Result<Option<f64>, ParseError> {
    let node = parse_str(expr_str)?;
    Ok(node.map(evaluate_node))
}

pub fn contains_operator(expr_str: &str) -> bool {
    for char in expr_str.chars() {
        if matches!(char, '+' | '-' | '*' | '/' | '^') {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {

    use crate::modules::parser::{self, Node, Operator};
    use std::boxed::Box;

    #[test]
    fn test_parser() -> Result<(), parser::ParseError> {
        let test_str = "1 * 2 - 1 + 3 / 2";
        let node = parser::parse_str(test_str)?.unwrap();
        let rhs = Node::Expr(
            Box::new(Node::Expr(
                Box::new(Node::Expr(
                    Box::new(Node::Number(1.0)),
                    Operator::Mult,
                    Box::new(Node::Number(2.0)),
                )),
                Operator::Sub,
                Box::new(Node::Number(1.0)),
            )),
            Operator::Add,
            Box::new(Node::Expr(
                Box::new(Node::Number(3.0)),
                Operator::Div,
                Box::new(Node::Number(2.0)),
            )),
        );

        assert_eq!(node, rhs);

        println!("GOT :: {:?}", node);
        let result = parser::evaluate_node(node);
        assert_eq!(result, 2.5);
        println!("RESULT :: {}", result);

        Ok(())
    }
}
