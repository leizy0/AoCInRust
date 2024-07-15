use std::{
    collections::HashMap,
    error,
    fmt::{Debug, Display},
    fs::File,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
};

use clap::Parser;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    InvalidExpChar(char),
    NonMatchingRightParen,
    TwoNeighborOp,
    MissingRightExp,
    MissingOperator(Box<dyn Expression>),
    EmptyExp,
    InvalidStartToken(Token),
    MissingRightParen,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::InvalidExpChar(c) => write!(f, "Invalid character for expression: {}", c),
            Error::NonMatchingRightParen => {
                write!(f, "Found non-matching right parenthesis in expression")
            }
            Error::TwoNeighborOp => write!(f, "Found two adjacent operators"),
            Error::MissingRightExp => write!(f, "Found an operator missing its right expression"),
            Error::MissingOperator(exp) => {
                write!(f, "Expect an operator after an subexpression({})", exp)
            }
            Error::EmptyExp => write!(f, "Given empty token stream"),
            Error::InvalidStartToken(t) => write!(f, "Invalid start token({:?}) for expression", t),
            Error::MissingRightParen => write!(
                f,
                "Expect a right parenthsis in an expression within parentheses"
            ),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug, Parser)]
pub struct CLIArgs {
    pub input_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
    Add,
    Mul,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    Num(usize),
    Op(Operator),
    LeftParen,
    RightParen,
}

pub trait Expression: Display + Debug {
    fn value(&self) -> usize;
}

#[derive(Debug)]
struct NumExp {
    n: usize,
}

impl Display for NumExp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.n)
    }
}

impl Expression for NumExp {
    fn value(&self) -> usize {
        self.n
    }
}

impl NumExp {
    pub fn new(n: usize) -> Self {
        Self { n }
    }
}

#[derive(Debug)]
struct AddExp {
    left: Box<dyn Expression>,
    right: Box<dyn Expression>,
}

impl Display for AddExp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} + {}", self.left, self.right)
    }
}

impl Expression for AddExp {
    fn value(&self) -> usize {
        self.left.value() + self.right.value()
    }
}

impl AddExp {
    pub fn new(left: Box<dyn Expression>, right: Box<dyn Expression>) -> Self {
        Self { left, right }
    }
}

#[derive(Debug)]
struct MulExp {
    left: Box<dyn Expression>,
    right: Box<dyn Expression>,
}

impl Display for MulExp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} * {}", self.left, self.right)
    }
}

impl Expression for MulExp {
    fn value(&self) -> usize {
        self.left.value() * self.right.value()
    }
}

impl MulExp {
    pub fn new(left: Box<dyn Expression>, right: Box<dyn Expression>) -> Self {
        Self { left, right }
    }
}

#[derive(Debug)]
struct ParenExp {
    inner: Box<dyn Expression>,
}

impl Display for ParenExp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.inner)
    }
}

impl Expression for ParenExp {
    fn value(&self) -> usize {
        self.inner.value()
    }
}

impl ParenExp {
    pub fn new(inner: Box<dyn Expression>) -> Self {
        Self { inner }
    }
}

struct TokenStream {
    tokens: Vec<Token>,
    ind: usize,
}

impl TryFrom<&str> for TokenStream {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut tokens = Vec::new();
        let mut number = None;
        for c in value.chars() {
            if c.is_ascii_digit() {
                number = Some(number.unwrap_or(0) * 10 + c.to_digit(10).unwrap() as usize);
            } else if c.is_ascii_whitespace() {
                // Ignore spaces.
                continue;
            } else {
                if let Some(n) = number.take() {
                    tokens.push(Token::Num(n));
                }

                tokens.push(match c {
                    '+' => Token::Op(Operator::Add),
                    '*' => Token::Op(Operator::Mul),
                    '(' => Token::LeftParen,
                    ')' => Token::RightParen,
                    other => return Err(Error::InvalidExpChar(other)),
                });
            }
        }

        if let Some(n) = number.take() {
            // Don't forget the last number.
            tokens.push(Token::Num(n));
        }

        Ok(Self { tokens, ind: 0 })
    }
}

impl TokenStream {
    pub fn pop(&mut self) -> Option<Token> {
        let ind = self.ind;
        self.ind += 1;
        self.tokens.get(ind).copied()
    }

    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.ind)
    }
}

pub fn read_exps<P: AsRef<Path>>(
    path: P,
    ops_prec: &HashMap<Operator, usize>,
) -> Result<Vec<Box<dyn Expression>>, Error> {
    let file = File::open(path).map_err(Error::IOError)?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .map(|l| {
            l.map_err(Error::IOError)
                .and_then(|s| parse_exp(s.as_str(), ops_prec))
        })
        .collect()
}

fn parse_exp(
    text: &str,
    ops_prec: &HashMap<Operator, usize>,
) -> Result<Box<dyn Expression>, Error> {
    let mut tokens = TokenStream::try_from(text)?;
    parse_exp_recur(&mut tokens, ops_prec, 0, false)
}

fn parse_exp_recur(
    tokens: &mut TokenStream,
    ops_prec: &HashMap<Operator, usize>,
    mut cur_prec: usize,
    in_paren: bool,
) -> Result<Box<dyn Expression>, Error> {
    let mut cur_exp: Option<Box<dyn Expression>> = None;
    while let Some(token) = tokens.peek().copied() {
        if let Some(exp) = cur_exp.take() {
            match token {
                Token::Op(op) => {
                    tokens.pop();
                    cur_prec = ops_prec[&op];
                    if let Some(next_token) = tokens.peek() {
                        let right_exp = match next_token {
                            Token::Num(_) => parse_exp_recur(tokens, ops_prec, cur_prec, in_paren)?,
                            Token::LeftParen => parse_paren_exp(tokens, ops_prec)?,
                            Token::RightParen => return Err(Error::NonMatchingRightParen),
                            Token::Op(_) => return Err(Error::TwoNeighborOp),
                        };

                        cur_exp = match op {
                            Operator::Add => Some(Box::new(AddExp::new(exp, right_exp))),
                            Operator::Mul => Some(Box::new(MulExp::new(exp, right_exp))),
                        };
                    } else {
                        return Err(Error::MissingRightExp);
                    }
                }
                Token::RightParen if in_paren => return Ok(exp),
                _ => return Err(Error::MissingOperator(exp)),
            }
        } else {
            match token {
                Token::Num(n) => {
                    tokens.pop();
                    cur_exp = Some(Box::new(NumExp::new(n)));
                    if let Some(Token::Op(op)) = tokens.peek() {
                        if ops_prec[&op] <= cur_prec {
                            break;
                        }
                    }
                }
                Token::LeftParen => cur_exp = Some(parse_paren_exp(tokens, ops_prec)?),
                other => return Err(Error::InvalidStartToken(other)),
            }
        }
    }

    cur_exp.ok_or(Error::EmptyExp)
}

fn parse_paren_exp(
    tokens: &mut TokenStream,
    ops_prec: &HashMap<Operator, usize>,
) -> Result<Box<dyn Expression>, Error> {
    assert!(tokens.pop().is_some_and(|t| t == Token::LeftParen));

    let exp = parse_exp_recur(tokens, ops_prec, 0, true)?;
    if let Some(Token::RightParen) = tokens.pop() {
        Ok(Box::new(ParenExp::new(exp)))
    } else {
        Err(Error::MissingRightParen)
    }
}
