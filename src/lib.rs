#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
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

//! The `wkt` crate provides conversions to and from [`WKT`](https://en.wikipedia.org/wiki/Well-known_text_representation_of_geometry) primitive types.
//! See the [`types`](crate::types) module for a list of available types.
//!
//! Conversions (using [`std::convert::From`] and [`std::convert::TryFrom`]) to and from [`geo_types`] primitives are enabled by default, but the feature is **optional**.
//!
//! Enable the `serde` feature if you need to deserialise data into custom structs containing `WKT` geometry fields.
//!
//! # Examples
//!
//! ```
//! use wkt::Wkt;
//! let point: Wkt<f64> = Wkt::from_str("POINT(10 20)").unwrap();
//! ```
//!
//! ```ignore
//! // Convert to a geo_types primitive from a Wkt struct
//! use std::convert::TryInto;
//! use wkt::Wkt;
//! use geo_types::Point;
//! let point: Wkt<f64> = Wkt::from_str("POINT(10 20)").unwrap();
//! let g_point: geo_types::Point<f64> = (10., 20.).into();
// // We can attempt to directly convert the Wkt without having to access its items field
//! let converted: Point<f64> = point.try_into().unwrap();
//! assert_eq!(g_point, converted);
//! ```
//!
//! ## Direct Access to the `item` Field
//! If you wish to work directly with one of the WKT [`types`] you can match on the `item` field
//! ```
//! use std::convert::TryInto;
//! use wkt::Wkt;
//! use wkt::Geometry;
//!
//! let wktls: Wkt<f64> = Wkt::from_str("LINESTRING(10 20, 20 30)").unwrap();
//! let ls = match wktls.item {
//!     Geometry::LineString(line_string) => {
//!         // you now have access to the types::LineString
//!     }
//!     _ => unreachable!(),
//! };
//!
//!
//!
use std::default::Default;
use std::fmt;
use std::str::FromStr;

use crate::tokenizer::{PeekableTokens, Token, Tokens};
use crate::types::GeometryCollection;
use crate::types::LineString;
use crate::types::MultiLineString;
use crate::types::MultiPoint;
use crate::types::MultiPolygon;
use crate::types::Point;
use crate::types::Polygon;

mod tokenizer;

#[cfg(feature = "geo-types")]
mod towkt;

/// `WKT` primitive types and collections
pub mod types;

#[cfg(feature = "geo-types")]
extern crate geo_types;

extern crate thiserror;

#[cfg(feature = "geo-types")]
pub use crate::towkt::ToWkt;

#[cfg(feature = "geo-types")]
pub mod conversion;

#[cfg(feature = "serde")]
extern crate serde;
#[cfg(feature = "serde")]
pub mod deserialize;
#[cfg(all(feature = "serde", feature = "geo-types"))]
pub use deserialize::{deserialize_geometry, deserialize_point};

pub trait WktFloat: num_traits::Float + std::fmt::Debug {}
impl<T> WktFloat for T where T: num_traits::Float + std::fmt::Debug {}

#[derive(Clone, Debug)]
/// All supported WKT geometry [`types`]
pub enum Geometry<T>
where
    T: WktFloat,
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
    T: WktFloat + FromStr + Default,
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
            w if w.eq_ignore_ascii_case("LINESTRING") || w.eq_ignore_ascii_case("LINEARRING") => {
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
    T: WktFloat + fmt::Display,
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

#[derive(Clone, Debug)]
/// Container for WKT primitives and collections
///
/// This type can be fallibly converted to a [`geo_types`] primitive using [`std::convert::TryFrom`].
pub struct Wkt<T>
where
    T: WktFloat,
{
    pub item: Geometry<T>,
}

impl<T> Wkt<T>
where
    T: WktFloat + FromStr + Default,
{
    pub fn from_str(wkt_str: &str) -> Result<Self, &'static str> {
        let tokens = Tokens::from_str(wkt_str);
        Wkt::from_tokens(tokens)
    }

    fn from_tokens(tokens: Tokens<T>) -> Result<Self, &'static str> {
        let mut tokens = tokens.peekable();
        let word = match tokens.next() {
            Some(Token::Word(word)) => {
                if !word.is_ascii() {
                    return Err("Encountered non-ascii word");
                }
                word
            }
            _ => return Err("Invalid WKT format"),
        };
        match Geometry::from_word_and_tokens(&word, &mut tokens) {
            Ok(item) => Ok(Wkt { item }),
            Err(s) => Err(s),
        }
    }
}

trait FromTokens<T>: Sized + Default
where
    T: WktFloat + FromStr + Default,
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

    fn from_tokens_with_optional_parens(
        tokens: &mut PeekableTokens<T>,
    ) -> Result<Self, &'static str> {
        match tokens.peek() {
            Some(Token::ParenOpen) => Self::from_tokens_with_parens(tokens),
            _ => Self::from_tokens(tokens),
        }
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
    use crate::types::{Coord, MultiPolygon, Point};
    use crate::{Geometry, Wkt};

    #[test]
    fn empty_string() {
        let res: Result<Wkt<f64>, _> = Wkt::from_str("");
        assert!(res.is_err());
    }

    #[test]
    fn empty_items() {
        let wkt: Wkt<f64> = Wkt::from_str("POINT EMPTY").ok().unwrap();
        match wkt.item {
            Geometry::Point(Point(None)) => (),
            _ => unreachable!(),
        };

        let wkt: Wkt<f64> = Wkt::from_str("MULTIPOLYGON EMPTY").ok().unwrap();
        match wkt.item {
            Geometry::MultiPolygon(MultiPolygon(polygons)) => assert_eq!(polygons.len(), 0),
            _ => unreachable!(),
        };
    }

    #[test]
    fn lowercase_point() {
        let wkt: Wkt<f64> = Wkt::from_str("point EMPTY").ok().unwrap();
        match wkt.item {
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

    #[test]
    fn support_jts_linearring() {
        let wkt: Wkt<f64> = Wkt::from_str("linearring (10 20, 30 40)").ok().unwrap();
        match wkt.item {
            Geometry::LineString(_ls) => (),
            _ => panic!("expected to be parsed as a LINESTRING"),
        };
    }

    #[test]
    fn test_debug() {
        let g = Geometry::Point(Point(Some(Coord {
            x: 1.0,
            y: 2.0,
            m: None,
            z: None,
        })));
        assert_eq!(
            format!("{:?}", g),
            "Point(Point(Some(Coord { x: 1.0, y: 2.0, z: None, m: None })))"
        );
    }
}
