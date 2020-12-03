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

use std::fmt;
use std::str::FromStr;
use tokenizer::PeekableTokens;
use types::linestring::LineString;
use FromTokens;
use Geometry;

#[derive(Clone, Default)]
pub struct Polygon<T: num_traits::Float>(pub Vec<LineString<T>>);

impl<T> Polygon<T>
where
    T: num_traits::Float,
{
    pub fn as_item(self) -> Geometry<T> {
        Geometry::Polygon(self)
    }
}

impl<T> fmt::Display for Polygon<T>
where
    T: num_traits::Float + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if self.0.is_empty() {
            f.write_str("POLYGON EMPTY")
        } else {
            let strings = self
                .0
                .iter()
                .map(|l| {
                    l.0.iter()
                        .map(|c| format!("{} {}", c.x, c.y))
                        .collect::<Vec<_>>()
                        .join(",")
                })
                .collect::<Vec<_>>()
                .join("),(");

            write!(f, "POLYGON(({}))", strings)
        }
    }
}

impl<T> FromTokens<T> for Polygon<T>
where
    T: num_traits::Float + FromStr + Default,
{
    fn from_tokens(tokens: &mut PeekableTokens<T>) -> Result<Self, &'static str> {
        let result = FromTokens::comma_many(
            <LineString<T> as FromTokens<T>>::from_tokens_with_parens,
            tokens,
        );
        result.map(Polygon)
    }
}

#[cfg(test)]
mod tests {
    use super::{LineString, Polygon};
    use types::Coord;
    use {Geometry, Wkt};

    #[test]
    fn basic_polygon() {
        let mut wkt: Wkt<f64> =
            Wkt::from_str("POLYGON ((8 4, 4 0, 0 4, 8 4), (7 3, 4 1, 1 4, 7 3))")
                .ok()
                .unwrap();
        assert_eq!(1, wkt.items.len());
        let lines = match wkt.items.pop().unwrap() {
            Geometry::Polygon(Polygon(lines)) => lines,
            _ => unreachable!(),
        };
        assert_eq!(2, lines.len());
    }

    #[test]
    fn write_empty_polygon() {
        let polygon: Polygon<f64> = Polygon(vec![]);

        assert_eq!("POLYGON EMPTY", format!("{}", polygon));
    }

    #[test]
    fn write_polygon() {
        let polygon = Polygon(vec![
            LineString(vec![
                Coord {
                    x: 0.,
                    y: 0.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 20.,
                    y: 40.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 40.,
                    y: 0.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 0.,
                    y: 0.,
                    z: None,
                    m: None,
                },
            ]),
            LineString(vec![
                Coord {
                    x: 5.,
                    y: 5.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 20.,
                    y: 30.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 30.,
                    y: 5.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 5.,
                    y: 5.,
                    z: None,
                    m: None,
                },
            ]),
        ]);

        assert_eq!(
            "POLYGON((0 0,20 40,40 0,0 0),(5 5,20 30,30 5,5 5))",
            format!("{}", polygon)
        );
    }
}
