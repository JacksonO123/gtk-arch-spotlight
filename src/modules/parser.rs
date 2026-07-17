use std::iter::Peekable;

#[derive(Debug)]
enum ParseError {
    InvalidNumber,
    UnexpectedCharacter(char),
    ExpectedTokenFoundNothing,
    UnexpectedToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Operator {
    Exponent,
    Mult,
    Div,
    Add,
    Sub,
}

#[derive(Debug, Clone)]
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
                res
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

    Ok(Some(Node::Expr(
        Box::new(unit_node),
        operator,
        Box::new(rhs),
    )))
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
                    return Err(ParseError::UnexpectedCharacter(char));
                }
            }
        };

        tokens.push(token);
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use crate::modules::parser::parse_str;

    #[test]
    fn test_parser() {
        let test_str = "2 + 2 / (4 * 3 ^ 1";
        let node = parse_str(test_str);
        println!("GOT :: {:?}", node);
    }
}
