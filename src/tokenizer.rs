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

    /*
    fn finish_number(&mut self) -> String {
    }
    */
}

#[test]
fn test_tokenizer_1word() {
    let test_str = "hello";
    let mut tokens = tokenize(test_str);
    match tokens.next().unwrap() {
        Token::Word(n) => assert_eq!(n, "hello".to_string()),
        _ => panic!("fail")
    }
    assert!(tokens.next().is_none());
}

#[test]
fn test_tokenizer_2words() {
    let test_str = "hello world";
    let mut tokens = tokenize(test_str);
    match tokens.next().unwrap() {
        Token::Word(n) => assert_eq!(n, "hello".to_string()),
        _ => panic!("fail")
    }
    match tokens.next().unwrap() {
        Token::Word(n) => assert_eq!(n, "world".to_string()),
        _ => panic!("fail")
    }
    assert!(tokens.next().is_none());
}

#[test]
fn test_tokenizer_1number() {
    let test_str = "4.2";
    let mut tokens = tokenize(test_str);
    match tokens.next().unwrap() {
        Token::Number(n) => assert_eq!(n.to_string(), "4.2"),
        _ => panic!("fail")
    }
    assert!(tokens.next().is_none());
}

#[test]
fn test_tokenizer_2numbers() {
    let test_str = ".38 -2";
    let mut tokens = tokenize(test_str);
    match tokens.next().unwrap() {
        Token::Number(n) => assert_eq!(n.to_string(), "0.38"),
        _ => panic!("fail")
    }
    match tokens.next().unwrap() {
        Token::Number(n) => assert_eq!(n.to_string(), "-2"),
        _ => panic!("fail")
    }
    assert!(tokens.next().is_none());
}
