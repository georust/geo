#[deriving(PartialEq, Show)]
pub enum Token {
    Comma,
    Number(f64),
    ParenClose,
    ParenOpen,
    Word(String),
}

pub fn tokenize(input: &str) -> Tokenizer {
    Tokenizer {text: String::from_str(input)}
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
        '.' | '-'           => true,
        _                   => false,
    }
}

struct Tokenizer {
    text: String
}

impl Iterator<Token> for Tokenizer {
    fn next(&mut self) -> Option<Token> {
        let popped_char = self.pop_char();
        if popped_char.is_none() {
            return None
        }
        let next_char = popped_char.unwrap();

        match next_char {
            '\0' => None,
            '(' => Some(Token::ParenOpen),
            ')' => Some(Token::ParenClose),
            ',' => Some(Token::Comma),
            c if is_whitespace(c) => self.next(),
            c if is_numberlike(c) => {
                let number = c.to_string() + self.read_until_whitespace().as_slice();
                let x: f64 = number.parse().unwrap();
                Some(Token::Number(x))
            }
            c => {
                let word = c.to_string() + self.read_until_whitespace().as_slice();
                Some(Token::Word(word))
            }
        }
    }
}

impl Tokenizer {
    fn pop_char(&mut self) -> Option<char> {
        if self.text.is_empty() {
            None
        } else {
            self.text.remove(0)
        }
    }

    fn read_until_whitespace(&mut self) -> String {
        let popped_char = self.pop_char();
        if popped_char.is_none() {
            return "".to_string()
        }
        let next_char = popped_char.unwrap();

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
    let tokens: Vec<Token> = tokenize(test_str).collect();
    assert_eq!(tokens, vec![]);
}

#[test]
fn test_tokenizer_1word() {
    let test_str = "hello";
    let tokens: Vec<Token> = tokenize(test_str).collect();
    assert_eq!(tokens, vec![
        Token::Word("hello".to_string()),
    ]);
}

#[test]
fn test_tokenizer_2words() {
    let test_str = "hello world";
    let tokens: Vec<Token> = tokenize(test_str).collect();
    assert_eq!(tokens, vec![
        Token::Word("hello".to_string()),
        Token::Word("world".to_string()),
    ]);
}

#[test]
fn test_tokenizer_1number() {
    let test_str = "4.2";
    let tokens: Vec<Token> = tokenize(test_str).collect();
    assert_eq!(tokens, vec![
        Token::Number(4.2),
    ]);
}

#[test]
fn test_tokenizer_2numbers() {
    let test_str = ".4 -2";
    let tokens: Vec<Token> = tokenize(test_str).collect();
    assert_eq!(tokens, vec![
        Token::Number(0.4),
        Token::Number(-2.0),
    ]);
}

#[test]
fn test_tokenizer_point() {
    let test_str = "POINT (10 -20)";
    let tokens: Vec<Token> = tokenize(test_str).collect();
    assert_eq!(tokens, vec![
        Token::Word("POINT".to_string()),
        Token::ParenOpen,
        Token::Number(10.0),
        Token::Number(-20.0),
        Token::ParenClose,
    ]);
}
