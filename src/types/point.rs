use tokenizer::PeekableTokens;
use types::FromTokens;
use types::coord::Coord;
use WktItem;


pub struct Point {
    pub coord: Coord
}

impl Point {
    pub fn as_item(self) -> WktItem {
        WktItem::Point(self)
    }
}

impl FromTokens for Point {
    fn from_tokens(tokens: &mut PeekableTokens) -> Result<Self, &'static str> {
        let coord = match Coord::from_tokens(tokens) {
            Ok(c) => c,
            Err(s) => return Err(s),
        };
        Ok(Point {coord: coord})
    }
}
