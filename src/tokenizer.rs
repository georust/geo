use std::iter::Peekable;

#[deriving(PartialEq, Show)]
pub enum Token {
    Comma,
    Number(f64),
    ParenClose,
    ParenOpen,
    Word(String),
}

fn is_whitespace(c: char) -> bool {
    match c {
        '\n' | '\r' | '\t' | ' ' => true,
        _                        => false,
    }
}

fn is_numberlike(c: char) -> bool {
    match c {
        c if c.is_numeric() => true,
        '.' | '-' | '+'     => true,
        _                   => false,
    }
}

pub type PeekableTokens = Peekable<Token, Tokens>;

pub struct Tokens {
    text: String
}

impl Tokens {
    pub fn from_str(input: &str) -> Self {
        Tokens {text: String::from_str(input)}
    }
}

impl Iterator<Token> for Tokens {
    fn next(&mut self) -> Option<Token> {
        let next_char = match self.pop_front() {
            Some(c) => c,
            None => return None,
        };

        match next_char {
            '\0' => None,
            '(' => Some(Token::ParenOpen),
            ')' => Some(Token::ParenClose),
            ',' => Some(Token::Comma),
            c if is_whitespace(c) => self.next(),
            c if is_numberlike(c) => {
                let mut number = c.to_string() + self.read_until_whitespace().as_slice();
                number = number.trim_left_matches('+').to_string();
                match number.parse::<f64>() {
                    Some(parsed_num) => Some(Token::Number(parsed_num)),
                    None => panic!("Could not parse number: {}", number),
                }
            }
            c => {
                let word = c.to_string() + self.read_until_whitespace().as_slice();
                Some(Token::Word(word))
            }
        }
    }
}

impl Tokens {
    fn pop_front(&mut self) -> Option<char> {
        match self.text.is_empty() {
            true => None,
            false => Some(self.text.remove(0))
        }
    }

    fn read_until_whitespace(&mut self) -> String {
        let next_char = match self.pop_front() {
            Some(c) => c,
            None => return "".to_string(),
        };

        match next_char {
            '\0' | '(' | ')' | ',' => {
                self.text.insert(0, next_char);
                "".to_string()
            }
            c if is_whitespace(c) => "".to_string(),
            _ => next_char.to_string() + self.read_until_whitespace().as_slice(),
        }
    }
}

#[test]
fn test_tokenizer_empty() {
    let test_str = "";
    let tokens: Vec<Token> = Tokens::from_str(test_str).collect();
    assert_eq!(tokens, vec![]);
}

#[test]
fn test_tokenizer_1word() {
    let test_str = "hello";
    let tokens: Vec<Token> = Tokens::from_str(test_str).collect();
    assert_eq!(tokens, vec![
        Token::Word("hello".to_string()),
    ]);
}

#[test]
fn test_tokenizer_2words() {
    let test_str = "hello world";
    let tokens: Vec<Token> = Tokens::from_str(test_str).collect();
    assert_eq!(tokens, vec![
        Token::Word("hello".to_string()),
        Token::Word("world".to_string()),
    ]);
}

#[test]
fn test_tokenizer_1number() {
    let test_str = "4.2";
    let tokens: Vec<Token> = Tokens::from_str(test_str).collect();
    assert_eq!(tokens, vec![
        Token::Number(4.2),
    ]);
}

#[test]
fn test_tokenizer_1number_plus() {
    let test_str = "+4.2";
    let tokens: Vec<Token> = Tokens::from_str(test_str).collect();
    assert_eq!(tokens, vec![
        Token::Number(4.2),
    ]);
}

#[test]
fn test_tokenizer_2numbers() {
    let test_str = ".4 -2";
    let tokens: Vec<Token> = Tokens::from_str(test_str).collect();
    assert_eq!(tokens, vec![
        Token::Number(0.4),
        Token::Number(-2.0),
    ]);
}

#[test]
fn test_tokenizer_point() {
    let test_str = "POINT (10 -20)";
    let tokens: Vec<Token> = Tokens::from_str(test_str).collect();
    assert_eq!(tokens, vec![
        Token::Word("POINT".to_string()),
        Token::ParenOpen,
        Token::Number(10.0),
        Token::Number(-20.0),
        Token::ParenClose,
    ]);
}
