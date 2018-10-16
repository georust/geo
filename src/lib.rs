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
use std::fmt;

use tokenizer::{PeekableTokens, Token, Tokens};
use types::GeometryCollection;
use types::LineString;
use types::MultiLineString;
use types::MultiPoint;
use types::MultiPolygon;
use types::Point;
use types::Polygon;

mod tokenizer;

#[cfg(feature = "geo-types")]
mod towkt;

pub mod types;

#[cfg(feature = "geo-types")]
pub use towkt::ToWkt;

#[cfg(feature = "geo-types")]
pub mod conversion;

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
    fn from_word_and_tokens(word: &str, tokens: &mut PeekableTokens) -> Result<Self, &'static str> {
        match word {
            w if w.eq_ignore_ascii_case("POINT") => {
                let x = <Point as FromTokens>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            }
            w if w.eq_ignore_ascii_case("LINESTRING") => {
                let x = <LineString as FromTokens>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            }
            w if w.eq_ignore_ascii_case("POLYGON") => {
                let x = <Polygon as FromTokens>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            }
            w if w.eq_ignore_ascii_case("MULTIPOINT") => {
                let x = <MultiPoint as FromTokens>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            }
            w if w.eq_ignore_ascii_case("MULTILINESTRING") => {
                let x = <MultiLineString as FromTokens>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            }
            w if w.eq_ignore_ascii_case("MULTIPOLYGON") => {
                let x = <MultiPolygon as FromTokens>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            }
            w if w.eq_ignore_ascii_case("GEOMETRYCOLLECTION") => {
                let x = <GeometryCollection as FromTokens>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            }
            _ => Err("Invalid type encountered"),
        }
    }
}

impl fmt::Display for Geometry {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Geometry::Point(point) => point.fmt(f),
            Geometry::LineString(linestring) => linestring.fmt(f),
            Geometry::Polygon(polygon) => polygon.fmt(f),
            Geometry::MultiPoint(multipoint) => multipoint.fmt(f),
            Geometry::MultiLineString(multilinstring) => multilinstring.fmt(f),
            Geometry::MultiPolygon(multipolygon) => multipolygon.fmt(f),
            Geometry::GeometryCollection(geometrycollection) => geometrycollection.fmt(f),
        }
    }
}

pub struct Wkt {
    pub items: Vec<Geometry>,
}

impl Wkt {
    pub fn new() -> Self {
        Wkt { items: vec![] }
    }

    pub fn add_item(&mut self, item: Geometry) {
        self.items.push(item);
    }

    pub fn from_str(wkt_str: &str) -> Result<Self, &'static str> {
        let tokens = Tokens::from_str(wkt_str);
        Wkt::from_tokens(tokens)
    }

    fn from_tokens(tokens: Tokens) -> Result<Self, &'static str> {
        let mut wkt = Wkt::new();
        let mut tokens = tokens.peekable();
        let word = match tokens.next() {
            Some(Token::Word(mut word)) => {
                if !word.is_ascii() {
                    return Err("Encountered non-ascii word");
                }
                word
            }
            None => return Ok(wkt),
            _ => return Err("Invalid WKT format"),
        };
        match Geometry::from_word_and_tokens(&word, &mut tokens) {
            Ok(item) => wkt.add_item(item),
            Err(s) => return Err(s),
        }
        Ok(wkt)
    }
}

trait FromTokens: Sized + Default {
    fn from_tokens(tokens: &mut PeekableTokens) -> Result<Self, &'static str>;

    fn from_tokens_with_parens(tokens: &mut PeekableTokens) -> Result<Self, &'static str> {
        match tokens.next() {
            Some(Token::ParenOpen) => (),
            Some(Token::Word(ref s)) if s.eq_ignore_ascii_case("EMPTY") => {
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

        let item = try!(f(tokens));
        items.push(item);

        while let Some(&Token::Comma) = tokens.peek() {
            tokens.next(); // throw away comma

            let item = try!(f(tokens));
            items.push(item);
        }

        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use types::{MultiPolygon, Point};
    use {Geometry, Wkt};

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
            Geometry::Point(Point(None)) => (),
            _ => unreachable!(),
        };

        let mut wkt = Wkt::from_str("MULTIPOLYGON EMPTY").ok().unwrap();
        assert_eq!(1, wkt.items.len());
        match wkt.items.pop().unwrap() {
            Geometry::MultiPolygon(MultiPolygon(polygons)) => assert_eq!(polygons.len(), 0),
            _ => unreachable!(),
        };
    }

    #[test]
    fn lowercase_point() {
        let mut wkt = Wkt::from_str("point EMPTY").ok().unwrap();
        assert_eq!(1, wkt.items.len());
        match wkt.items.pop().unwrap() {
            Geometry::Point(Point(None)) => (),
            _ => unreachable!(),
        };
    }
}
