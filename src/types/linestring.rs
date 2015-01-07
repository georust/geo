use tokenizer::{PeekableTokens, Token};
use types::FromTokens;
use types::coord::Coord;
use WktItem;


pub struct LineString {
    pub coords: Vec<Coord>
}

impl LineString {
    pub fn as_item(self) -> WktItem {
        WktItem::LineString(self)
    }
}

impl FromTokens for LineString {
    fn from_tokens(tokens: &mut PeekableTokens) -> Result<Self, &'static str> {
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
}
