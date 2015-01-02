use tokenizer::{Token, Tokenizer};
use types::coord::Coord;


pub struct Point {
    pub coord: Coord
}

impl Point {
    pub fn from_tokens(tokens: &mut Tokenizer) ->  Result<Self, &'static str> {
        match tokens.next() {
            Some(Token::ParenOpen) => (),
            _ => return Err("Missing open parenthesis for POINT"),
        };
        let coord = match Coord::from_tokens(tokens) {
            Ok(c) => c,
            Err(s) => return Err(s),
        };
        match tokens.next() {
            Some(Token::ParenClose) => (),
            _ => return Err("Missing closing parenthesis for POINT"),
        };
        Ok(Point {coord: coord})
    }
}
