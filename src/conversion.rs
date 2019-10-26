// Copyright 2014-2018 The GeoRust Developers
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

extern crate geo_types;
extern crate num_traits;

use std::fmt;
use types::*;
use Geometry;

#[derive(Debug)]
pub enum Error {
    PointConversionError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::PointConversionError => {
                f.write_str("impossible to convert empty point to geo_type point")
            }
        }
    }
}

fn create_geo_coordinate<T>(coord: &Coord<T>) -> geo_types::Coordinate<T>
where
    T: num_traits::Float,
{
    geo_types::Coordinate {
        x: T::from(coord.x).unwrap(),
        y: T::from(coord.y).unwrap(),
    }
}

fn try_into_point<T>(point: &Point<T>) -> Result<geo_types::Geometry<T>, Error>
where
    T: num_traits::Float,
{
    match point.0 {
        Some(ref c) => {
            let geo_point: geo_types::Point<T> = (c.x, c.y).into();
            Ok(geo_point.into())
        }
        None => Err(Error::PointConversionError),
    }
}

pub fn try_into_geometry<T>(geometry: &Geometry<T>) -> Result<geo_types::Geometry<T>, Error>
where
    T: num_traits::Float,
{
    match geometry {
        Geometry::Point(point) => try_into_point(point),
        Geometry::LineString(linestring) => Ok(linestring.into()),
        Geometry::Polygon(polygon) => Ok(polygon.into()),
        Geometry::MultiLineString(multilinestring) => Ok(multilinestring.into()),
        Geometry::MultiPoint(multipoint) => Ok(multipoint.into()),
        Geometry::MultiPolygon(multipolygon) => Ok(multipolygon.into()),
        Geometry::GeometryCollection(geometrycollection) => {
            try_into_geometry_collection(geometrycollection)
        }
    }
}

impl<'a, T> Into<geo_types::Geometry<T>> for &'a LineString<T>
where
    T: num_traits::Float,
{
    fn into(self) -> geo_types::Geometry<T> {
        let geo_linestring: geo_types::LineString<T> =
            self.0.iter().map(|c| create_geo_coordinate(&c)).collect();

        geo_linestring.into()
    }
}

impl<'a, T> Into<geo_types::Geometry<T>> for &'a MultiLineString<T>
where
    T: num_traits::Float,
{
    fn into(self) -> geo_types::Geometry<T> {
        let geo_multilinestring: geo_types::MultiLineString<T> = self
            .0
            .iter()
            .map(|l| {
                l.0.iter()
                    .map(|c| create_geo_coordinate(&c))
                    .collect::<Vec<_>>()
            })
            .collect();

        geo_multilinestring.into()
    }
}

fn w_polygon_to_g_polygon<T>(polygon: &Polygon<T>) -> geo_types::Polygon<T>
where
    T: num_traits::Float,
{
    let mut iter = polygon.0.iter().map(|l| {
        l.0.iter()
            .map(|c| create_geo_coordinate(c))
            .collect::<Vec<_>>()
            .into()
    });

    match iter.next() {
        Some(interior) => geo_types::Polygon::new(interior, iter.collect()),
        None => geo_types::Polygon::new(geo_types::LineString(vec![]), vec![]),
    }
}

impl<'a, T> Into<geo_types::Geometry<T>> for &'a Polygon<T>
where
    T: num_traits::Float,
{
    fn into(self) -> geo_types::Geometry<T> {
        w_polygon_to_g_polygon(self).into()
    }
}

impl<'a, T> Into<geo_types::Geometry<T>> for &'a MultiPoint<T>
where
    T: num_traits::Float,
{
    fn into(self) -> geo_types::Geometry<T> {
        let geo_multipoint: geo_types::MultiPoint<T> = self
            .0
            .iter()
            .filter_map(|p| p.0.as_ref())
            .map(|c| create_geo_coordinate(c))
            .collect();

        geo_multipoint.into()
    }
}

impl<'a, T> Into<geo_types::Geometry<T>> for &'a MultiPolygon<T>
where
    T: num_traits::Float,
{
    fn into(self) -> geo_types::Geometry<T> {
        let geo_multipolygon: geo_types::MultiPolygon<T> =
            self.0.iter().map(|p| w_polygon_to_g_polygon(p)).collect();

        geo_multipolygon.into()
    }
}

pub fn try_into_geometry_collection<T>(
    geometrycollection: &GeometryCollection<T>,
) -> Result<geo_types::Geometry<T>, Error>
where
    T: num_traits::Float,
{
    let geo_geometrycollection: geo_types::GeometryCollection<T> = geometrycollection
        .0
        .iter()
        .map(|g| try_into_geometry(g))
        .collect::<Result<_, _>>()?;

    Ok(geo_types::Geometry::GeometryCollection(
        geo_geometrycollection,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_empty_point() {
        let point = Point(None).as_item();
        let res: Result<geo_types::Geometry<f64>, Error> = try_into_geometry(&point);
        assert!(res.is_err());
    }

    #[test]
    fn convert_point() {
        let point = Point(Some(Coord {
            x: 10.,
            y: 20.,
            z: None,
            m: None,
        }))
        .as_item();

        let g_point: geo_types::Point<f64> = (10., 20.).into();
        assert_eq!(
            geo_types::Geometry::Point(g_point),
            try_into_geometry(&point).unwrap()
        );
    }

    #[test]
    fn convert_empty_linestring() {
        let w_linestring = LineString(vec![]).as_item();
        let g_linestring: geo_types::LineString<f64> = geo_types::LineString(vec![]);
        assert_eq!(
            geo_types::Geometry::LineString(g_linestring),
            try_into_geometry(&w_linestring).unwrap()
        );
    }

    #[test]
    fn convert_linestring() {
        let w_linestring = LineString(vec![
            Coord {
                x: 10.,
                y: 20.,
                z: None,
                m: None,
            },
            Coord {
                x: 30.,
                y: 40.,
                z: None,
                m: None,
            },
        ])
        .as_item();
        let g_linestring: geo_types::LineString<f64> = vec![(10., 20.), (30., 40.)].into();
        assert_eq!(
            geo_types::Geometry::LineString(g_linestring),
            try_into_geometry(&w_linestring).unwrap()
        );
    }

    #[test]
    fn convert_empty_polygon() {
        let w_polygon = Polygon(vec![]).as_item();
        let g_polygon: geo_types::Polygon<f64> =
            geo_types::Polygon::new(geo_types::LineString(vec![]), vec![]);
        assert_eq!(
            geo_types::Geometry::Polygon(g_polygon),
            try_into_geometry(&w_polygon).unwrap()
        );
    }

    #[test]
    fn convert_polygon() {
        let w_polygon = Polygon(vec![
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
        ])
        .as_item();
        let g_polygon: geo_types::Polygon<f64> = geo_types::Polygon::new(
            vec![(0., 0.), (20., 40.), (40., 0.), (0., 0.)].into(),
            vec![vec![(5., 5.), (20., 30.), (30., 5.), (5., 5.)].into()],
        );
        assert_eq!(
            geo_types::Geometry::Polygon(g_polygon),
            try_into_geometry(&w_polygon).unwrap()
        );
    }

    #[test]
    fn convert_empty_multilinestring() {
        let w_multilinestring = MultiLineString(vec![]).as_item();
        let g_multilinestring: geo_types::MultiLineString<f64> = geo_types::MultiLineString(vec![]);
        assert_eq!(
            geo_types::Geometry::MultiLineString(g_multilinestring),
            try_into_geometry(&w_multilinestring).unwrap()
        );
    }

    #[test]
    fn convert_multilinestring() {
        let w_multilinestring = MultiLineString(vec![
            LineString(vec![
                Coord {
                    x: 10.,
                    y: 20.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 30.,
                    y: 40.,
                    z: None,
                    m: None,
                },
            ]),
            LineString(vec![
                Coord {
                    x: 50.,
                    y: 60.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 70.,
                    y: 80.,
                    z: None,
                    m: None,
                },
            ]),
        ])
        .as_item();
        let g_multilinestring: geo_types::MultiLineString<f64> = geo_types::MultiLineString(vec![
            vec![(10., 20.), (30., 40.)].into(),
            vec![(50., 60.), (70., 80.)].into(),
        ]);
        assert_eq!(
            geo_types::Geometry::MultiLineString(g_multilinestring),
            try_into_geometry(&w_multilinestring).unwrap()
        );
    }

    #[test]
    fn convert_empty_multipoint() {
        let w_multipoint = MultiPoint(vec![]).as_item();
        let g_multipoint: geo_types::MultiPoint<f64> = geo_types::MultiPoint(vec![]);
        assert_eq!(
            geo_types::Geometry::MultiPoint(g_multipoint),
            try_into_geometry(&w_multipoint).unwrap()
        );
    }

    #[test]
    fn convert_multipoint() {
        let w_multipoint = MultiPoint(vec![
            Point(Some(Coord {
                x: 10.,
                y: 20.,
                z: None,
                m: None,
            })),
            Point(Some(Coord {
                x: 30.,
                y: 40.,
                z: None,
                m: None,
            })),
        ])
        .as_item();
        let g_multipoint: geo_types::MultiPoint<f64> = vec![(10., 20.), (30., 40.)].into();
        assert_eq!(
            geo_types::Geometry::MultiPoint(g_multipoint),
            try_into_geometry(&w_multipoint).unwrap()
        );
    }

    #[test]
    fn convert_empty_multipolygon() {
        let w_multipolygon = MultiPolygon(vec![]).as_item();
        let g_multipolygon: geo_types::MultiPolygon<f64> = geo_types::MultiPolygon(vec![]);
        assert_eq!(
            geo_types::Geometry::MultiPolygon(g_multipolygon),
            try_into_geometry(&w_multipolygon).unwrap()
        );
    }

    #[test]
    fn convert_multipolygon() {
        let w_multipolygon = MultiPolygon(vec![
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
        ])
        .as_item();

        let g_multipolygon: geo_types::MultiPolygon<f64> = geo_types::MultiPolygon(vec![
            geo_types::Polygon::new(
                vec![(0., 0.), (20., 40.), (40., 0.), (0., 0.)].into(),
                vec![vec![(5., 5.), (20., 30.), (30., 5.), (5., 5.)].into()],
            ),
            geo_types::Polygon::new(
                vec![(40., 40.), (20., 45.), (45., 30.), (40., 40.)].into(),
                vec![],
            ),
        ]);
        assert_eq!(
            geo_types::Geometry::MultiPolygon(g_multipolygon),
            try_into_geometry(&w_multipolygon).unwrap()
        );
    }

    #[test]
    fn convert_empty_geometrycollection() {
        let w_geometrycollection = GeometryCollection(vec![]).as_item();
        let g_geometrycollection: geo_types::GeometryCollection<f64> =
            geo_types::GeometryCollection(vec![]);
        assert_eq!(
            geo_types::Geometry::GeometryCollection(g_geometrycollection),
            try_into_geometry(&w_geometrycollection).unwrap()
        );
    }

    #[test]
    fn convert_geometrycollection() {
        let w_point = Point(Some(Coord {
            x: 10.,
            y: 20.,
            z: None,
            m: None,
        }))
        .as_item();

        let w_linestring = LineString(vec![
            Coord {
                x: 10.,
                y: 20.,
                z: None,
                m: None,
            },
            Coord {
                x: 30.,
                y: 40.,
                z: None,
                m: None,
            },
        ])
        .as_item();

        let w_polygon = Polygon(vec![LineString(vec![
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
        ])])
        .as_item();

        let w_multilinestring = MultiLineString(vec![
            LineString(vec![
                Coord {
                    x: 10.,
                    y: 20.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 30.,
                    y: 40.,
                    z: None,
                    m: None,
                },
            ]),
            LineString(vec![
                Coord {
                    x: 50.,
                    y: 60.,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 70.,
                    y: 80.,
                    z: None,
                    m: None,
                },
            ]),
        ])
        .as_item();

        let w_multipoint = MultiPoint(vec![
            Point(Some(Coord {
                x: 10.,
                y: 20.,
                z: None,
                m: None,
            })),
            Point(Some(Coord {
                x: 30.,
                y: 40.,
                z: None,
                m: None,
            })),
        ])
        .as_item();

        let w_multipolygon = MultiPolygon(vec![
            Polygon(vec![LineString(vec![
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
            ])]),
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
        ])
        .as_item();

        let w_geometrycollection = GeometryCollection(vec![
            w_point,
            w_multipoint,
            w_linestring,
            w_multilinestring,
            w_polygon,
            w_multipolygon,
        ])
        .as_item();

        let g_point: geo_types::Point<f64> = (10., 20.).into();
        let g_linestring: geo_types::LineString<f64> = vec![(10., 20.), (30., 40.)].into();
        let g_polygon: geo_types::Polygon<f64> = geo_types::Polygon::new(
            vec![(0., 0.), (20., 40.), (40., 0.), (0., 0.)].into(),
            vec![],
        );
        let g_multilinestring: geo_types::MultiLineString<f64> = geo_types::MultiLineString(vec![
            vec![(10., 20.), (30., 40.)].into(),
            vec![(50., 60.), (70., 80.)].into(),
        ]);
        let g_multipoint: geo_types::MultiPoint<f64> = vec![(10., 20.), (30., 40.)].into();
        let g_multipolygon: geo_types::MultiPolygon<f64> = geo_types::MultiPolygon(vec![
            geo_types::Polygon::new(
                vec![(0., 0.), (20., 40.), (40., 0.), (0., 0.)].into(),
                vec![],
            ),
            geo_types::Polygon::new(
                vec![(40., 40.), (20., 45.), (45., 30.), (40., 40.)].into(),
                vec![],
            ),
        ]);

        let g_geometrycollection: geo_types::GeometryCollection<f64> =
            geo_types::GeometryCollection(vec![
                geo_types::Geometry::Point(g_point),
                geo_types::Geometry::MultiPoint(g_multipoint),
                geo_types::Geometry::LineString(g_linestring),
                geo_types::Geometry::MultiLineString(g_multilinestring),
                geo_types::Geometry::Polygon(g_polygon),
                geo_types::Geometry::MultiPolygon(g_multipolygon),
            ]);
        assert_eq!(
            geo_types::Geometry::GeometryCollection(g_geometrycollection),
            try_into_geometry(&w_geometrycollection).unwrap()
        );
    }
}
