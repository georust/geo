// Copyright 2014-2015 The GeoRust Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::iter::Peekable;
use std::marker::PhantomData;
use std::str;
use WktFloat;

#[derive(Debug, PartialEq)]
pub enum Token<T>
where
    T: WktFloat,
{
    Comma,
    Number(T),
    ParenClose,
    ParenOpen,
    Word(String),
}

#[inline]
fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\n' || c == '\r' || c == '\t'
}

#[inline]
fn is_numberlike(c: char) -> bool {
    c == '.' || c == '-' || c == '+' || c.is_ascii_digit()
}

pub type PeekableTokens<'a, T> = Peekable<Tokens<'a, T>>;

#[derive(Debug)]
pub struct Tokens<'a, T> {
    chars: Peekable<str::Chars<'a>>,
    phantom: PhantomData<T>,
}

impl<'a, T> Tokens<'a, T>
where
    T: WktFloat,
{
    pub fn from_str(input: &'a str) -> Self {
        Tokens {
            chars: input.chars().peekable(),
            phantom: PhantomData,
        }
    }
}

impl<'a, T> Iterator for Tokens<'a, T>
where
    T: WktFloat + str::FromStr,
{
    type Item = Token<T>;

    fn next(&mut self) -> Option<Token<T>> {
        // TODO: should this return Result?
        let mut next_char = self.chars.next()?;

        // Skip whitespace
        while is_whitespace(next_char) {
            next_char = self.chars.next()?
        }

        match next_char {
            '\0' => None,
            '(' => Some(Token::ParenOpen),
            ')' => Some(Token::ParenClose),
            ',' => Some(Token::Comma),
            c if is_numberlike(c) => {
                let mut number = self.read_until_whitespace().unwrap_or_default();
                if c != '+' {
                    // Prepend the character because the string likely has capacity.
                    number.insert(0, c);
                }
                match number.parse::<T>() {
                    Ok(parsed_num) => Some(Token::Number(parsed_num)),
                    Err(_) => None,
                }
            }
            c => {
                let mut word = self.read_until_whitespace().unwrap_or_default();
                word.insert(0, c);
                Some(Token::Word(word))
            }
        }
    }
}

impl<'a, T> Tokens<'a, T>
where
    T: str::FromStr,
{
    fn read_until_whitespace(&mut self) -> Option<String> {
        let mut result = None;

        while let Some(&next_char) = self.chars.peek() {
            match next_char {
                '\0' | '(' | ')' | ',' => break, // Just stop on a marker
                c if is_whitespace(c) => {
                    let _ = self.chars.next();
                    break;
                }
                _ => {
                    let _ = self.chars.next();
                    result
                        .get_or_insert_with(|| String::with_capacity(16))
                        .push(next_char);
                }
            }
        }

        result
    }
}

#[test]
fn test_tokenizer_empty() {
    let test_str = "";
    let tokens: Vec<Token<f64>> = Tokens::from_str(test_str).collect();
    assert_eq!(tokens, vec![]);
}

#[test]
fn test_tokenizer_1word() {
    let test_str = "hello";
    let tokens: Vec<Token<f64>> = Tokens::from_str(test_str).collect();
    assert_eq!(tokens, vec![Token::Word("hello".to_string())]);
}

#[test]
fn test_tokenizer_2words() {
    let test_str = "hello world";
    let tokens: Vec<Token<f64>> = Tokens::from_str(test_str).collect();
    assert_eq!(
        tokens,
        vec![
            Token::Word("hello".to_string()),
            Token::Word("world".to_string()),
        ]
    );
}

#[test]
fn test_tokenizer_1number() {
    let test_str = "4.2";
    let tokens: Vec<Token<f64>> = Tokens::from_str(test_str).collect();
    assert_eq!(tokens, vec![Token::Number(4.2)]);
}

#[test]
fn test_tokenizer_1number_plus() {
    let test_str = "+4.2";
    let tokens: Vec<Token<f64>> = Tokens::from_str(test_str).collect();
    assert_eq!(tokens, vec![Token::Number(4.2)]);
}

#[test]
fn test_tokenizer_invalid_number() {
    let test_str = "4.2p";
    let tokens: Vec<Token<f64>> = Tokens::from_str(test_str).collect();
    assert_eq!(tokens, vec![]);
}

#[test]
fn test_tokenizer_not_a_number() {
    let test_str = "¾"; // A number according to char.is_numeric()
    let tokens: Vec<Token<f64>> = Tokens::from_str(test_str).collect();
    assert_eq!(tokens, vec![Token::Word("¾".to_owned())]);
}

#[test]
fn test_tokenizer_2numbers() {
    let test_str = ".4 -2";
    let tokens: Vec<Token<f64>> = Tokens::from_str(test_str).collect();
    assert_eq!(tokens, vec![Token::Number(0.4), Token::Number(-2.0)]);
}

#[test]
fn test_no_stack_overflow() {
    fn check(c: &str, count: usize, expected: usize) {
        let test_str = c.repeat(count);
        let tokens: Vec<Token<f64>> = Tokens::from_str(&test_str).collect();
        assert_eq!(expected, tokens.len());
    }

    let count = 100_000;
    check("+", count, 0);
    check(" ", count, 0);
    check("A", count, 1);
    check("1", count, 1);
    check("(", count, count);
    check(")", count, count);
    check(",", count, count);
}

#[test]
fn test_tokenizer_point() {
    let test_str = "POINT (10 -20)";
    let tokens: Vec<Token<f64>> = Tokens::from_str(test_str).collect();
    assert_eq!(
        tokens,
        vec![
            Token::Word("POINT".to_string()),
            Token::ParenOpen,
            Token::Number(10.0),
            Token::Number(-20.0),
            Token::ParenClose,
        ]
    );
}
