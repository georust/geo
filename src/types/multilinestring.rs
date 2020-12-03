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

extern crate num_traits;

use std::fmt;
use std::str::FromStr;
use tokenizer::PeekableTokens;
use types::linestring::LineString;
use FromTokens;
use Geometry;

#[derive(Clone, Default)]
pub struct MultiLineString<T: num_traits::Float>(pub Vec<LineString<T>>);

impl<T> MultiLineString<T>
where
    T: num_traits::Float,
{
    pub fn as_item(self) -> Geometry<T> {
        Geometry::MultiLineString(self)
    }
}

impl<T> fmt::Display for MultiLineString<T>
where
    T: num_traits::Float + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if self.0.is_empty() {
            f.write_str("MULTILINESTRING EMPTY")
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

            write!(f, "MULTILINESTRING(({}))", strings)
        }
    }
}

impl<T> FromTokens<T> for MultiLineString<T>
where
    T: num_traits::Float + FromStr + Default,
{
    fn from_tokens(tokens: &mut PeekableTokens<T>) -> Result<Self, &'static str> {
        let result = FromTokens::comma_many(
            <LineString<T> as FromTokens<T>>::from_tokens_with_parens,
            tokens,
        );
        result.map(MultiLineString)
    }
}

#[cfg(test)]
mod tests {
    use super::{LineString, MultiLineString};
    use types::Coord;
    use {Geometry, Wkt};

    #[test]
    fn basic_multilinestring() {
        let mut wkt: Wkt<f64> = Wkt::from_str("MULTILINESTRING ((8 4, -3 0), (4 0, 6 -10))")
            .ok()
            .unwrap();
        assert_eq!(1, wkt.items.len());
        let lines = match wkt.items.pop().unwrap() {
            Geometry::MultiLineString(MultiLineString(lines)) => lines,
            _ => unreachable!(),
        };
        assert_eq!(2, lines.len());
    }

    #[test]
    fn write_empty_multilinestring() {
        let multilinestring: MultiLineString<f64> = MultiLineString(vec![]);

        assert_eq!("MULTILINESTRING EMPTY", format!("{}", multilinestring));
    }

    #[test]
    fn write_multilinestring() {
        let multilinestring = MultiLineString(vec![
            LineString(vec![
                Coord {
                    x: 10.1,
                    y: 20.2,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 30.3,
                    y: 40.4,
                    z: None,
                    m: None,
                },
            ]),
            LineString(vec![
                Coord {
                    x: 50.5,
                    y: 60.6,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 70.7,
                    y: 80.8,
                    z: None,
                    m: None,
                },
            ]),
        ]);

        assert_eq!(
            "MULTILINESTRING((10.1 20.2,30.3 40.4),(50.5 60.6,70.7 80.8))",
            format!("{}", multilinestring)
        );
    }
}
