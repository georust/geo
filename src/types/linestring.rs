use std::iter::Peekable;

use tokenizer::{Token, Tokenizer};
use types::coord::Coord;


pub struct LineString {
    pub coords: Vec<Coord>
}

impl LineString {
    pub fn from_tokens(tokens: &mut Peekable<Token, Tokenizer>) ->  Result<Self, &'static str> {
        let mut coords = Vec::new();

        match tokens.next() {
            Some(Token::ParenOpen) => (),
            _ => return Err("Missing open parenthesis for LINESTRING"),
        };
        coords.push(match Coord::from_tokens(tokens) {
            Ok(c) => c,
            Err(s) => return Err(s),
        });

        while let Some(&Token::Comma) = tokens.peek() {
            tokens.next();  // throw away comma

            coords.push(match Coord::from_tokens(tokens) {
                Ok(c) => c,
                Err(s) => return Err(s),
            });
        }

        match tokens.next() {
            Some(Token::ParenClose) => (),
            _ => return Err("Missing closing parenthesis for LINESTRING"),
        };
        Ok(LineString {coords: coords})
    }
}
