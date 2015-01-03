use std::iter::Peekable;

use tokenizer::{Token, Tokenizer};
use types::coord::Coord;
use WktItem;


pub struct LineString {
    pub coords: Vec<Coord>
}

impl LineString {
    pub fn from_tokens(tokens: &mut Peekable<Token, Tokenizer>) -> Result<Self, &'static str> {
        let mut coords = Vec::new();

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

        Ok(LineString {coords: coords})
    }

    pub fn as_item(self) -> WktItem {
        WktItem::LineString(self)
    }
}
