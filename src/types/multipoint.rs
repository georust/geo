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
use types::point::Point;
use FromTokens;
use Geometry;

#[derive(Clone, Default)]
pub struct MultiPoint<T: num_traits::Float>(pub Vec<Point<T>>);

impl<T> MultiPoint<T>
where
    T: num_traits::Float,
{
    pub fn as_item(self) -> Geometry<T> {
        Geometry::MultiPoint(self)
    }
}

impl<T> fmt::Display for MultiPoint<T>
where
    T: num_traits::Float + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if self.0.is_empty() {
            f.write_str("MULTIPOINT EMPTY")
        } else {
            let strings = self
                .0
                .iter()
                .filter_map(|p| p.0.as_ref())
                .map(|c| format!("({} {})", c.x, c.y))
                .collect::<Vec<_>>()
                .join(",");

            write!(f, "MULTIPOINT({})", strings)
        }
    }
}

impl<T> FromTokens<T> for MultiPoint<T>
where
    T: num_traits::Float + FromStr + Default,
{
    fn from_tokens(tokens: &mut PeekableTokens<T>) -> Result<Self, &'static str> {
        let result =
            FromTokens::comma_many(<Point<T> as FromTokens<T>>::from_tokens_with_parens, tokens);
        result.map(MultiPoint)
    }
}

#[cfg(test)]
mod tests {
    use super::{MultiPoint, Point};
    use types::Coord;
    use {Geometry, Wkt};

    #[test]
    fn basic_multipoint() {
        let mut wkt: Wkt<f64> = Wkt::from_str("MULTIPOINT ((8 4), (4 0))").ok().unwrap();
        assert_eq!(1, wkt.items.len());
        let points = match wkt.items.pop().unwrap() {
            Geometry::MultiPoint(MultiPoint(points)) => points,
            _ => unreachable!(),
        };
        assert_eq!(2, points.len());
    }

    #[test]
    fn write_empty_multipoint() {
        let multipoint: MultiPoint<f64> = MultiPoint(vec![]);

        assert_eq!("MULTIPOINT EMPTY", format!("{}", multipoint));
    }

    #[test]
    fn write_multipoint() {
        let multipoint = MultiPoint(vec![
            Point(Some(Coord {
                x: 10.1,
                y: 20.2,
                z: None,
                m: None,
            })),
            Point(Some(Coord {
                x: 30.3,
                y: 40.4,
                z: None,
                m: None,
            })),
        ]);

        assert_eq!(
            "MULTIPOINT((10.1 20.2),(30.3 40.4))",
            format!("{}", multipoint)
        );
    }
}
