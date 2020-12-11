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
use types::coord::Coord;
use FromTokens;
use Geometry;

#[derive(Clone, Default)]
pub struct LineString<T: num_traits::Float>(pub Vec<Coord<T>>);

impl<T> LineString<T>
where
    T: num_traits::Float,
{
    pub fn as_item(self) -> Geometry<T> {
        Geometry::LineString(self)
    }
}

impl<T> FromTokens<T> for LineString<T>
where
    T: num_traits::Float + FromStr + Default,
{
    fn from_tokens(tokens: &mut PeekableTokens<T>) -> Result<Self, &'static str> {
        let result = FromTokens::comma_many(<Coord<T> as FromTokens<T>>::from_tokens, tokens);
        result.map(LineString)
    }
}

impl<T> fmt::Display for LineString<T>
where
    T: num_traits::Float + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if self.0.is_empty() {
            f.write_str("LINESTRING EMPTY")
        } else {
            let strings = self
                .0
                .iter()
                .map(|c| format!("{}", c))
                .collect::<Vec<_>>()
                .join(",");

            write!(f, "LINESTRING({})", strings)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Coord, LineString};
    use {Geometry, Wkt};

    #[test]
    fn basic_linestring() {
        let mut wkt = Wkt::from_str("LINESTRING (10 -20, -0 -0.5)").ok().unwrap();
        assert_eq!(1, wkt.items.len());
        let coords = match wkt.items.pop().unwrap() {
            Geometry::LineString(LineString(coords)) => coords,
            _ => unreachable!(),
        };
        assert_eq!(2, coords.len());

        assert_eq!(10.0, coords[0].x);
        assert_eq!(-20.0, coords[0].y);
        assert_eq!(None, coords[0].z);
        assert_eq!(None, coords[0].m);

        assert_eq!(0.0, coords[1].x);
        assert_eq!(-0.5, coords[1].y);
        assert_eq!(None, coords[1].z);
        assert_eq!(None, coords[1].m);
    }

    #[test]
    fn write_empty_linestring() {
        let linestring: LineString<f64> = LineString(vec![]);

        assert_eq!("LINESTRING EMPTY", format!("{}", linestring));
    }

    #[test]
    fn write_linestring() {
        let linestring = LineString(vec![
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
        ]);

        assert_eq!("LINESTRING(10.1 20.2,30.3 40.4)", format!("{}", linestring));
    }
}
