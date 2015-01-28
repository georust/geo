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

#![allow(unstable)]

use std::ascii::AsciiExt;

use tokenizer::{PeekableTokens, Token, Tokens};
use types::FromTokens;
use types::geometrycollection::GeometryCollection;
use types::linestring::LineString;
use types::point::Point;
use types::polygon::Polygon;
use types::multipoint::MultiPoint;
use types::multilinestring::MultiLineString;
use types::multipolygon::MultiPolygon;

mod tokenizer;
mod types;


pub enum Geometry {
    Point(Point),
    LineString(LineString),
    Polygon(Polygon),
    MultiPoint(MultiPoint),
    MultiLineString(MultiLineString),
    MultiPolygon(MultiPolygon),
    GeometryCollection(GeometryCollection),
}

impl Geometry {
    fn from_word_and_tokens(word: &str, tokens: &mut PeekableTokens)-> Result<Self, &'static str> {
        match word {
            "POINT" => {
                let x = <Point as FromTokens>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            },
            "LINESTRING" => {
                let x = <LineString as FromTokens>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            },
            "POLYGON" => {
                let x = <Polygon as FromTokens>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            },
            "MULTIPOINT" => {
                let x = <MultiPoint as FromTokens>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            },
            "MULTILINESTRING" => {
                let x = <MultiLineString as FromTokens>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            },
            "MULTIPOLYGON" => {
                let x = <MultiPolygon as FromTokens>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            },
            "GEOMETRYCOLLECTION" => {
                let x = <GeometryCollection as FromTokens>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            },
            _ => Err("Invalid type encountered"),
        }
    }
}


pub struct Wkt {
    items: Vec<Geometry>
}

impl Wkt {
    fn new() -> Self {
        Wkt {items: vec![]}
    }

    fn add_item(&mut self, item: Geometry) {
        self.items.push(item);
    }

    fn from_str(wkt_str: &str) -> Result<Self, &'static str> {
        let tokens = Tokens::from_str(wkt_str);
        Wkt::from_tokens(tokens)
    }

    fn from_tokens(tokens: Tokens) -> Result<Self, &'static str> {
        let mut wkt = Wkt::new();
        let mut tokens = tokens.peekable();
        let word = match tokens.next() {
            Some(Token::Word(word)) => {
                if !word.is_ascii() {
                    return Err("Encountered non-ascii word");
                }
                word.to_ascii_uppercase()
            },
            None => return Ok(wkt),
            _ => return Err("Invalid WKT format"),
        };
        match Geometry::from_word_and_tokens(word.as_slice(), &mut tokens) {
            Ok(item) => wkt.add_item(item),
            Err(s) => return Err(s),
        }
        Ok(wkt)
    }
}


#[cfg(test)]
mod tests {
    use super::{Wkt, Geometry};
    use super::types::multipolygon::MultiPolygon;
    use super::types::point::Point;

    #[test]
    fn empty_string() {
        let wkt = Wkt::from_str("").ok().unwrap();
        assert_eq!(0, wkt.items.len());
    }

    #[test]
    fn empty_items() {
        let mut wkt = Wkt::from_str("POINT EMPTY").ok().unwrap();
        assert_eq!(1, wkt.items.len());
        match wkt.items.pop().unwrap() {
            Geometry::Point(Point { coord: None }) => (),
            _ => unreachable!(),
        };

        let mut wkt = Wkt::from_str("MULTIPOLYGON EMPTY").ok().unwrap();
        assert_eq!(1, wkt.items.len());
        match wkt.items.pop().unwrap() {
            Geometry::MultiPolygon(MultiPolygon { polygons }) =>
                assert_eq!(polygons.len(), 0),
            _ => unreachable!(),
        };
    }
}
