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

use crate::tokenizer::PeekableTokens;
use crate::types::linestring::LineString;
use crate::{FromTokens, Geometry, WktFloat};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, Default)]
pub struct Polygon<T: WktFloat>(pub Vec<LineString<T>>);

impl<T> Polygon<T>
where
    T: WktFloat,
{
    pub fn as_item(self) -> Geometry<T> {
        Geometry::Polygon(self)
    }
}

impl<T> fmt::Display for Polygon<T>
where
    T: WktFloat + fmt::Display,
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
    T: WktFloat + FromStr + Default,
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
    use crate::types::Coord;
    use crate::{Geometry, Wkt};
    use std::str::FromStr;

    #[test]
    fn basic_polygon() {
        let wkt: Wkt<f64> = Wkt::from_str("POLYGON ((8 4, 4 0, 0 4, 8 4), (7 3, 4 1, 1 4, 7 3))")
            .ok()
            .unwrap();
        let lines = match wkt.item {
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
