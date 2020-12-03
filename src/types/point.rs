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
pub struct Point<T: num_traits::Float>(pub Option<Coord<T>>);

impl<T> Point<T>
where
    T: num_traits::Float,
{
    pub fn as_item(self) -> Geometry<T> {
        Geometry::Point(self)
    }
}

impl<T> fmt::Display for Point<T>
where
    T: num_traits::Float + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self.0 {
            Some(ref coord) => {
                let mut lrs = String::new();
                if coord.z.is_some() {
                    lrs += "Z";
                }
                if coord.m.is_some() {
                    lrs += "M";
                }
                if !lrs.is_empty() {
                    lrs = " ".to_string() + &lrs;
                }

                write!(f, "POINT{}({})", lrs, coord)
            }
            None => f.write_str("POINT EMPTY"),
        }
    }
}

impl<T> FromTokens<T> for Point<T>
where
    T: num_traits::Float + FromStr + Default,
{
    fn from_tokens(tokens: &mut PeekableTokens<T>) -> Result<Self, &'static str> {
        let result = <Coord<T> as FromTokens<T>>::from_tokens(tokens);
        result.map(|coord| Point(Some(coord)))
    }
}

#[cfg(test)]
mod tests {
    use super::{Coord, Point};
    use {Geometry, Wkt};

    #[test]
    fn basic_point() {
        let mut wkt = Wkt::from_str("POINT (10 -20)").ok().unwrap();
        assert_eq!(1, wkt.items.len());
        let coord = match wkt.items.pop().unwrap() {
            Geometry::Point(Point(Some(coord))) => coord,
            _ => unreachable!(),
        };
        assert_eq!(10.0, coord.x);
        assert_eq!(-20.0, coord.y);
        assert_eq!(None, coord.z);
        assert_eq!(None, coord.m);
    }

    #[test]
    fn basic_point_whitespace() {
        let mut wkt: Wkt<f64> =
            Wkt::from_str(" \n\t\rPOINT \n\t\r( \n\r\t10 \n\t\r-20 \n\t\r) \n\t\r")
                .ok()
                .unwrap();
        assert_eq!(1, wkt.items.len());
        let coord = match wkt.items.pop().unwrap() {
            Geometry::Point(Point(Some(coord))) => coord,
            _ => unreachable!(),
        };
        assert_eq!(10.0, coord.x);
        assert_eq!(-20.0, coord.y);
        assert_eq!(None, coord.z);
        assert_eq!(None, coord.m);
    }

    #[test]
    fn invalid_points() {
        <Wkt<f64>>::from_str("POINT ()").err().unwrap();
        <Wkt<f64>>::from_str("POINT (10)").err().unwrap();
        <Wkt<f64>>::from_str("POINT 10").err().unwrap();
        <Wkt<f64>>::from_str("POINT (10 -20 40)").err().unwrap();
    }

    #[test]
    fn write_empty_point() {
        let point: Point<f64> = Point(None);

        assert_eq!("POINT EMPTY", format!("{}", point));
    }

    #[test]
    fn write_2d_point() {
        let point = Point(Some(Coord {
            x: 10.12345,
            y: 20.67891,
            z: None,
            m: None,
        }));

        assert_eq!("POINT(10.12345 20.67891)", format!("{}", point));
    }

    #[test]
    fn write_point_with_z_coord() {
        let point = Point(Some(Coord {
            x: 10.12345,
            y: 20.67891,
            z: Some(-32.56455),
            m: None,
        }));

        assert_eq!("POINT Z(10.12345 20.67891 -32.56455)", format!("{}", point));
    }

    #[test]
    fn write_point_with_m_coord() {
        let point = Point(Some(Coord {
            x: 10.12345,
            y: 20.67891,
            z: None,
            m: Some(10.),
        }));

        assert_eq!("POINT M(10.12345 20.67891 10)", format!("{}", point));
    }

    #[test]
    fn write_point_with_zm_coord() {
        let point = Point(Some(Coord {
            x: 10.12345,
            y: 20.67891,
            z: Some(-32.56455),
            m: Some(10.),
        }));

        assert_eq!(
            "POINT ZM(10.12345 20.67891 -32.56455 10)",
            format!("{}", point)
        );
    }
}
