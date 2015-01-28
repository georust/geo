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

use tokenizer::PeekableTokens;
use types::FromTokens;
use types::coord::Coord;
use Geometry;


#[derive(Default)]
pub struct Point {
    pub coord: Option<Coord>
}

impl Point {
    pub fn as_item(self) -> Geometry {
        Geometry::Point(self)
    }
}

impl FromTokens for Point {
    fn from_tokens(tokens: &mut PeekableTokens) -> Result<Self, &'static str> {
        let result = <Coord as FromTokens>::from_tokens(tokens);
        result.map(|coord| Point {coord: Some(coord)})
    }
}


#[cfg(test)]
mod tests {
    use {Wkt, Geometry};

    #[test]
    fn basic_point() {
        let mut wkt = Wkt::from_str("POINT (10 -20)").ok().unwrap();
        assert_eq!(1, wkt.items.len());
        let coord = match wkt.items.pop().unwrap() {
            Geometry::Point(point) => point.coord.unwrap(),
            _ => unreachable!(),
        };
        assert_eq!(10.0, coord.x);
        assert_eq!(-20.0, coord.y);
        assert_eq!(None, coord.z);
        assert_eq!(None, coord.m);
    }

    #[test]
    fn basic_point_whitespace() {
        let mut wkt = Wkt::from_str(" \n\t\rPOINT \n\t\r( \n\r\t10 \n\t\r-20 \n\t\r) \n\t\r").ok().unwrap();
        assert_eq!(1, wkt.items.len());
        let coord = match wkt.items.pop().unwrap() {
            Geometry::Point(point) => point.coord.unwrap(),
            _ => unreachable!(),
        };
        assert_eq!(10.0, coord.x);
        assert_eq!(-20.0, coord.y);
        assert_eq!(None, coord.z);
        assert_eq!(None, coord.m);
    }

    #[test]
    fn invalid_points() {
        Wkt::from_str("POINT ()").err().unwrap();
        Wkt::from_str("POINT (10)").err().unwrap();
        Wkt::from_str("POINT 10").err().unwrap();
        Wkt::from_str("POINT (10 -20 40)").err().unwrap();
    }
}
