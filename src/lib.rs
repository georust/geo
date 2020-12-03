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

extern crate num_traits;

use std::default::Default;
use std::fmt;
use std::str::FromStr;

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

#[derive(Clone)]
pub enum Geometry<T>
where
    T: num_traits::Float,
{
    Point(Point<T>),
    LineString(LineString<T>),
    Polygon(Polygon<T>),
    MultiPoint(MultiPoint<T>),
    MultiLineString(MultiLineString<T>),
    MultiPolygon(MultiPolygon<T>),
    GeometryCollection(GeometryCollection<T>),
}

impl<T> Geometry<T>
where
    T: num_traits::Float + FromStr + Default,
{
    fn from_word_and_tokens(
        word: &str,
        tokens: &mut PeekableTokens<T>,
    ) -> Result<Self, &'static str> {
        match word {
            w if w.eq_ignore_ascii_case("POINT") => {
                let x = <Point<T> as FromTokens<T>>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            }
            w if w.eq_ignore_ascii_case("LINESTRING") => {
                let x = <LineString<T> as FromTokens<T>>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            }
            w if w.eq_ignore_ascii_case("POLYGON") => {
                let x = <Polygon<T> as FromTokens<T>>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            }
            w if w.eq_ignore_ascii_case("MULTIPOINT") => {
                let x = <MultiPoint<T> as FromTokens<T>>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            }
            w if w.eq_ignore_ascii_case("MULTILINESTRING") => {
                let x = <MultiLineString<T> as FromTokens<T>>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            }
            w if w.eq_ignore_ascii_case("MULTIPOLYGON") => {
                let x = <MultiPolygon<T> as FromTokens<T>>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            }
            w if w.eq_ignore_ascii_case("GEOMETRYCOLLECTION") => {
                let x = <GeometryCollection<T> as FromTokens<T>>::from_tokens_with_parens(tokens);
                x.map(|y| y.as_item())
            }
            _ => Err("Invalid type encountered"),
        }
    }
}

impl<T> fmt::Display for Geometry<T>
where
    T: num_traits::Float + fmt::Display,
{
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

#[derive(Clone)]
pub struct Wkt<T>
where
    T: num_traits::Float,
{
    pub items: Vec<Geometry<T>>,
}

impl<T> Wkt<T>
where
    T: num_traits::Float + FromStr + Default,
{
    pub fn new() -> Self {
        Wkt { items: vec![] }
    }

    pub fn add_item(&mut self, item: Geometry<T>) {
        self.items.push(item);
    }

    pub fn from_str(wkt_str: &str) -> Result<Self, &'static str> {
        let tokens = Tokens::from_str(wkt_str);
        Wkt::from_tokens(tokens)
    }

    fn from_tokens(tokens: Tokens<T>) -> Result<Self, &'static str> {
        let mut wkt = Wkt::new();
        let mut tokens = tokens.peekable();
        let word = match tokens.next() {
            Some(Token::Word(word)) => {
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

trait FromTokens<T>: Sized + Default
where
    T: num_traits::Float + FromStr + Default,
{
    fn from_tokens(tokens: &mut PeekableTokens<T>) -> Result<Self, &'static str>;

    fn from_tokens_with_parens(tokens: &mut PeekableTokens<T>) -> Result<Self, &'static str> {
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

    fn comma_many<F>(f: F, tokens: &mut PeekableTokens<T>) -> Result<Vec<Self>, &'static str>
    where
        F: Fn(&mut PeekableTokens<T>) -> Result<Self, &'static str>,
    {
        let mut items = Vec::new();

        let item = f(tokens)?;
        items.push(item);

        while let Some(&Token::Comma) = tokens.peek() {
            tokens.next(); // throw away comma

            let item = f(tokens)?;
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
        let wkt: Wkt<f64> = Wkt::from_str("").ok().unwrap();
        assert_eq!(0, wkt.items.len());
    }

    #[test]
    fn empty_items() {
        let mut wkt: Wkt<f64> = Wkt::from_str("POINT EMPTY").ok().unwrap();
        assert_eq!(1, wkt.items.len());
        match wkt.items.pop().unwrap() {
            Geometry::Point(Point(None)) => (),
            _ => unreachable!(),
        };

        let mut wkt: Wkt<f64> = Wkt::from_str("MULTIPOLYGON EMPTY").ok().unwrap();
        assert_eq!(1, wkt.items.len());
        match wkt.items.pop().unwrap() {
            Geometry::MultiPolygon(MultiPolygon(polygons)) => assert_eq!(polygons.len(), 0),
            _ => unreachable!(),
        };
    }

    #[test]
    fn lowercase_point() {
        let mut wkt: Wkt<f64> = Wkt::from_str("point EMPTY").ok().unwrap();
        assert_eq!(1, wkt.items.len());
        match wkt.items.pop().unwrap() {
            Geometry::Point(Point(None)) => (),
            _ => unreachable!(),
        };
    }

    #[test]
    fn invalid_number() {
        if let Err(msg) = <Wkt<f64>>::from_str("POINT (10 20.1A)") {
            assert_eq!("Expected a number for the Y coordinate", msg);
        } else {
            panic!("Should not have parsed");
        }
    }
}
