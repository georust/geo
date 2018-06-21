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

use std::fmt;
use tokenizer::PeekableTokens;
use types::polygon::Polygon;
use FromTokens;
use Geometry;

#[derive(Default)]
pub struct MultiPolygon(pub Vec<Polygon>);

impl MultiPolygon {
    pub fn as_item(self) -> Geometry {
        Geometry::MultiPolygon(self)
    }
}

impl fmt::Display for MultiPolygon {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if self.0.is_empty() {
            f.write_str("MULTIPOLYGON EMPTY")
        } else {
            let strings = self
                .0
                .iter()
                .map(|p| {
                    p.0
                        .iter()
                        .map(|l| {
                            l.0
                                .iter()
                                .map(|c| format!("{} {}", c.x, c.y))
                                .collect::<Vec<String>>()
                                .join(",")
                        })
                        .collect::<Vec<String>>()
                        .join("),(")
                })
                .collect::<Vec<String>>()
                .join(")),((");

            write!(f, "MULTIPOLYGON((({})))", strings)
        }
    }
}

impl FromTokens for MultiPolygon {
    fn from_tokens(tokens: &mut PeekableTokens) -> Result<Self, &'static str> {
        let result =
            FromTokens::comma_many(<Polygon as FromTokens>::from_tokens_with_parens, tokens);
        result.map(MultiPolygon)
    }
}

#[cfg(test)]
mod tests {
    use super::{MultiPolygon, Polygon};
    use types::{Coord, LineString};
    use {Geometry, Wkt};

    #[test]
    fn basic_multipolygon() {
        let mut wkt = Wkt::from_str("MULTIPOLYGON (((8 4)), ((4 0)))")
            .ok()
            .unwrap();
        assert_eq!(1, wkt.items.len());
        let polygons = match wkt.items.pop().unwrap() {
            Geometry::MultiPolygon(MultiPolygon(polygons)) => polygons,
            _ => unreachable!(),
        };
        assert_eq!(2, polygons.len());
    }

    #[test]
    fn write_empty_multipolygon() {
        let multipolygon = MultiPolygon(vec![]);

        assert_eq!("MULTIPOLYGON EMPTY", format!("{}", multipolygon));
    }

    #[test]
    fn write_multipolygon() {
        let multipolygon = MultiPolygon(vec![
            Polygon(vec![
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
            ]),
            Polygon(vec![LineString(vec![
                Coord {
                    x: 40.,
                    y: 40.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 20.,
                    y: 45.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 45.,
                    y: 30.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 40.,
                    y: 40.,
                    z: None,
                    m: None,
                },
            ])]),
        ]);

        assert_eq!(
            "MULTIPOLYGON(((0 0,20 40,40 0,0 0),(5 5,20 30,30 5,5 5)),((40 40,20 45,45 30,40 40)))",
            format!("{}", multipolygon)
        );
    }
}
