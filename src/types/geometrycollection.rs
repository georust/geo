// Copyright 2015 The GeoRust Developers
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

use tokenizer::{PeekableTokens, Token};
use FromTokens;
use Geometry;

#[derive(Default)]
pub struct GeometryCollection(pub Vec<Geometry>);

impl GeometryCollection {
    pub fn as_item(self) -> Geometry {
        Geometry::GeometryCollection(self)
    }
}

impl FromTokens for GeometryCollection {
    fn from_tokens(tokens: &mut PeekableTokens) -> Result<Self, &'static str> {
        let mut items = Vec::new();

        let word = match tokens.next() {
            Some(Token::Word(w)) => w,
            _ => return Err("Expected a word in GEOMETRYCOLLECTION"),
        };

        let item = try!(Geometry::from_word_and_tokens(&*word, tokens));
        items.push(item);

        while let Some(&Token::Comma) = tokens.peek() {
            tokens.next(); // throw away comma

            let word = match tokens.next() {
                Some(Token::Word(w)) => w,
                _ => return Err("Expected a word in GEOMETRYCOLLECTION"),
            };

            let item = try!(Geometry::from_word_and_tokens(&*word, tokens));
            items.push(item);
        }

        Ok(GeometryCollection(items))
    }
}



#[cfg(test)]
mod tests {
    use {Wkt, Geometry};
    use super::GeometryCollection;

    #[test]
    fn basic_geometrycollection() {
        let mut wkt = Wkt::from_str("GEOMETRYCOLLECTION (POINT (8 4)))").ok().unwrap();
        assert_eq!(1, wkt.items.len());
        let items = match wkt.items.pop().unwrap() {
            Geometry::GeometryCollection(GeometryCollection(items)) => items,
            _ => unreachable!(),
        };
        assert_eq!(1, items.len());
    }
}
