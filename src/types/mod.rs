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

use std::default::Default;

use tokenizer::{PeekableTokens, Token};

pub use self::coord::Coord;
pub use self::geometrycollection::GeometryCollection;
pub use self::linestring::LineString;
pub use self::multilinestring::MultiLineString;
pub use self::multipoint::MultiPoint;
pub use self::multipolygon::MultiPolygon;
pub use self::point::Point;
pub use self::polygon::Polygon;

mod coord;
mod geometrycollection;
mod linestring;
mod multilinestring;
mod multipoint;
mod multipolygon;
mod point;
mod polygon;

trait FromTokens: Sized + Default {
    fn from_tokens(tokens: &mut PeekableTokens) -> Result<Self, &'static str>;

    fn from_tokens_with_parens(tokens: &mut PeekableTokens) -> Result<Self, &'static str> {
        match tokens.next() {
            Some(Token::ParenOpen) => (),
            Some(Token::Word(ref s)) if s.to_ascii_uppercase() == "EMPTY" => {
                return Ok(Default::default())
            }
            _ => return Err("Missing open parenthesis for type"),
        };
        let result = FromTokens::from_tokens(tokens);
        match tokens.next() {
            Some(Token::ParenClose) => (),
            _ => return Err("Missing closing parenthesis for type"),
        };
        result
    }

    fn comma_many<F>(f: F, tokens: &mut PeekableTokens) -> Result<Vec<Self>, &'static str>
    where
        F: Fn(&mut PeekableTokens) -> Result<Self, &'static str>,
    {
        let mut items = Vec::new();

        match f(tokens) {
            Ok(i) => items.push(i),
            Err(s) => return Err(s),
        };

        while let Some(&Token::Comma) = tokens.peek() {
            tokens.next(); // throw away comma

            match f(tokens) {
                Ok(i) => items.push(i),
                Err(s) => return Err(s),
            };
        }

        Ok(items)
    }
}
