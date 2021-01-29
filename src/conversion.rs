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

use types::*;
use Geometry;
use Wkt;

use std::convert::{TryFrom, TryInto};

use geo_types::CoordFloat;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("The WKT Point was empty, but geo_type::Points cannot be empty")]
    PointConversionError,
    #[error("Mismatched geometry (expected {expected:?}, found {found:?})")]
    MismatchedGeometry {
        expected: &'static str,
        found: &'static str,
    },
    #[error("Wrong number of Geometries: {0}")]
    WrongNumberOfGeometries(usize),
    #[error("External error: {0}")]
    External(Box<dyn std::error::Error>),
}

impl<T> TryFrom<Wkt<T>> for geo_types::Geometry<T>
where
    T: CoordFloat,
{
    type Error = Error;

    fn try_from(mut wkt: Wkt<T>) -> Result<Self, Self::Error> {
        if wkt.items.len() == 1 {
            Self::try_from(wkt.items.pop().unwrap())
        } else {
            Geometry::GeometryCollection(GeometryCollection(wkt.items)).try_into()
        }
    }
}

#[macro_use]
macro_rules! try_from_wkt_impl {
    ($($type: ident),+) => {
        $(
            /// Convert a Wkt enum into a specific geo-type
            impl<T: CoordFloat> TryFrom<Wkt<T>> for geo_types::$type<T> {
                type Error = Error;

                fn try_from(mut wkt: Wkt<T>) -> Result<Self, Self::Error> {
                    match wkt.items.len() {
                        1 => {
                            let item = wkt.items.pop().unwrap();
                            let geometry = geo_types::Geometry::try_from(item)?;
                            Self::try_from(geometry).map_err(|e| {
                                match e {
                                    geo_types::Error::MismatchedGeometry { expected, found } => {
                                        Error::MismatchedGeometry { expected, found }
                                    }
                                    // currently only one error type in geo-types error enum, but that seems likely to change
                                    #[allow(unreachable_patterns)]
                                    other => Error::External(Box::new(other)),
                                }
                            })
                        }
                        other => Err(Error::WrongNumberOfGeometries(other)),
                    }
                }
            }
        )+
    }
}

try_from_wkt_impl!(
    Point,
    Line,
    LineString,
    Polygon,
    MultiPoint,
    MultiLineString,
    MultiPolygon,
    Rect,
    Triangle
);

impl<T> From<Coord<T>> for geo_types::Coordinate<T>
where
    T: CoordFloat,
{
    fn from(coord: Coord<T>) -> geo_types::Coordinate<T> {
        Self {
            x: coord.x,
            y: coord.y,
        }
    }
}

impl<T> TryFrom<Point<T>> for geo_types::Point<T>
where
    T: CoordFloat,
{
    type Error = Error;

    fn try_from(point: Point<T>) -> Result<Self, Self::Error> {
        match point.0 {
            Some(coord) => Ok(Self::new(coord.x, coord.y)),
            None => Err(Error::PointConversionError),
        }
    }
}

#[deprecated(since = "0.9", note = "use `geometry.try_into()` instead")]
pub fn try_into_geometry<T>(geometry: &Geometry<T>) -> Result<geo_types::Geometry<T>, Error>
where
    T: CoordFloat,
{
    geometry.clone().try_into()
}

impl<'a, T> From<&'a LineString<T>> for geo_types::Geometry<T>
where
    T: CoordFloat,
{
    fn from(line_string: &'a LineString<T>) -> Self {
        Self::LineString(line_string.clone().into())
    }
}

impl<T> From<LineString<T>> for geo_types::LineString<T>
where
    T: CoordFloat,
{
    fn from(line_string: LineString<T>) -> Self {
        let coords = line_string
            .0
            .into_iter()
            .map(geo_types::Coordinate::from)
            .collect();

        geo_types::LineString(coords)
    }
}

impl<'a, T> From<&'a MultiLineString<T>> for geo_types::Geometry<T>
where
    T: CoordFloat,
{
    fn from(multi_line_string: &'a MultiLineString<T>) -> geo_types::Geometry<T> {
        Self::MultiLineString(multi_line_string.clone().into())
    }
}

impl<T> From<MultiLineString<T>> for geo_types::MultiLineString<T>
where
    T: CoordFloat,
{
    fn from(multi_line_string: MultiLineString<T>) -> geo_types::MultiLineString<T> {
        let geo_line_strings: Vec<geo_types::LineString<T>> = multi_line_string
            .0
            .into_iter()
            .map(geo_types::LineString::from)
            .collect();

        geo_types::MultiLineString(geo_line_strings)
    }
}

impl<'a, T> From<&'a Polygon<T>> for geo_types::Geometry<T>
where
    T: CoordFloat,
{
    fn from(polygon: &'a Polygon<T>) -> geo_types::Geometry<T> {
        Self::Polygon(polygon.clone().into())
    }
}

impl<T> From<Polygon<T>> for geo_types::Polygon<T>
where
    T: CoordFloat,
{
    fn from(polygon: Polygon<T>) -> Self {
        let mut iter = polygon.0.into_iter().map(geo_types::LineString::from);
        match iter.next() {
            Some(interior) => geo_types::Polygon::new(interior, iter.collect()),
            None => geo_types::Polygon::new(geo_types::LineString(vec![]), vec![]),
        }
    }
}

impl<'a, T> TryFrom<&'a MultiPoint<T>> for geo_types::Geometry<T>
where
    T: CoordFloat,
{
    type Error = Error;

    fn try_from(multi_point: &'a MultiPoint<T>) -> Result<Self, Self::Error> {
        Ok(Self::MultiPoint(multi_point.clone().try_into()?))
    }
}

impl<T> TryFrom<MultiPoint<T>> for geo_types::MultiPoint<T>
where
    T: CoordFloat,
{
    type Error = Error;

    fn try_from(multi_point: MultiPoint<T>) -> Result<Self, Self::Error> {
        let points: Vec<geo_types::Point<T>> = multi_point
            .0
            .into_iter()
            .map(geo_types::Point::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(geo_types::MultiPoint(points))
    }
}

impl<'a, T> From<&'a MultiPolygon<T>> for geo_types::Geometry<T>
where
    T: CoordFloat,
{
    fn from(multi_polygon: &'a MultiPolygon<T>) -> Self {
        Self::MultiPolygon(multi_polygon.clone().into())
    }
}

impl<T> From<MultiPolygon<T>> for geo_types::MultiPolygon<T>
where
    T: CoordFloat,
{
    fn from(multi_polygon: MultiPolygon<T>) -> Self {
        let geo_polygons: Vec<geo_types::Polygon<T>> = multi_polygon
            .0
            .into_iter()
            .map(geo_types::Polygon::from)
            .collect();

        geo_types::MultiPolygon(geo_polygons)
    }
}

#[deprecated(since = "0.9", note = "use `geometry_collection.try_into()` instead")]
pub fn try_into_geometry_collection<T>(
    geometry_collection: &GeometryCollection<T>,
) -> Result<geo_types::Geometry<T>, Error>
where
    T: CoordFloat,
{
    Ok(geo_types::Geometry::GeometryCollection(
        geometry_collection.clone().try_into()?,
    ))
}

impl<T> TryFrom<GeometryCollection<T>> for geo_types::GeometryCollection<T>
where
    T: CoordFloat,
{
    type Error = Error;

    fn try_from(geometry_collection: GeometryCollection<T>) -> Result<Self, Self::Error> {
        let geo_geometeries = geometry_collection
            .0
            .into_iter()
            .map(Geometry::try_into)
            .collect::<Result<_, _>>()?;

        Ok(geo_types::GeometryCollection(geo_geometeries))
    }
}

impl<T> TryFrom<Geometry<T>> for geo_types::Geometry<T>
where
    T: CoordFloat,
{
    type Error = Error;

    fn try_from(geometry: Geometry<T>) -> Result<Self, Self::Error> {
        Ok(match geometry {
            Geometry::Point(g) => geo_types::Geometry::Point(g.try_into()?),
            Geometry::LineString(g) => geo_types::Geometry::LineString(g.into()),
            Geometry::Polygon(g) => geo_types::Geometry::Polygon(g.into()),
            Geometry::MultiLineString(g) => geo_types::Geometry::MultiLineString(g.into()),
            Geometry::MultiPoint(g) => geo_types::Geometry::MultiPoint(g.try_into()?),
            Geometry::MultiPolygon(g) => geo_types::Geometry::MultiPolygon(g.into()),
            Geometry::GeometryCollection(g) => {
                geo_types::Geometry::GeometryCollection(g.try_into()?)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_single_item_wkt() {
        let w_point = Point(Some(Coord {
            x: 1.0,
            y: 2.0,
            z: None,
            m: None,
        }))
        .as_item();
        let mut wkt = Wkt::new();
        wkt.add_item(w_point);

        let converted = geo_types::Geometry::try_from(wkt).unwrap();
        let g_point: geo_types::Point<f64> = geo_types::Point::new(1.0, 2.0);

        assert_eq!(converted, geo_types::Geometry::Point(g_point));
    }

    #[test]
    fn convert_collection_wkt() {
        let w_point_1 = Point(Some(Coord {
            x: 1.0,
            y: 2.0,
            z: None,
            m: None,
        }))
        .as_item();
        let w_point_2 = Point(Some(Coord {
            x: 3.0,
            y: 4.0,
            z: None,
            m: None,
        }))
        .as_item();
        let mut wkt = Wkt::new();
        wkt.add_item(w_point_1);
        wkt.add_item(w_point_2);

        let converted = geo_types::Geometry::try_from(wkt).unwrap();
        let geo_collection: geo_types::GeometryCollection<f64> =
            geo_types::GeometryCollection(vec![
                geo_types::Point::new(1.0, 2.0).into(),
                geo_types::Point::new(3.0, 4.0).into(),
            ]);

        assert_eq!(
            converted,
            geo_types::Geometry::GeometryCollection(geo_collection)
        );
    }

    #[test]
    fn convert_empty_point() {
        let point = Point(None).as_item();
        let res: Result<geo_types::Geometry<f64>, Error> = point.try_into();
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
            point.try_into().unwrap()
        );
    }

    #[test]
    fn convert_empty_linestring() {
        let w_linestring = LineString(vec![]).as_item();
        let g_linestring: geo_types::LineString<f64> = geo_types::LineString(vec![]);
        assert_eq!(
            geo_types::Geometry::LineString(g_linestring),
            w_linestring.try_into().unwrap()
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
            w_linestring.try_into().unwrap()
        );
    }

    #[test]
    fn convert_empty_polygon() {
        let w_polygon = Polygon(vec![]).as_item();
        let g_polygon: geo_types::Polygon<f64> =
            geo_types::Polygon::new(geo_types::LineString(vec![]), vec![]);
        assert_eq!(
            geo_types::Geometry::Polygon(g_polygon),
            w_polygon.try_into().unwrap()
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
            w_polygon.try_into().unwrap()
        );
    }

    #[test]
    fn convert_empty_multilinestring() {
        let w_multilinestring = MultiLineString(vec![]).as_item();
        let g_multilinestring: geo_types::MultiLineString<f64> = geo_types::MultiLineString(vec![]);
        assert_eq!(
            geo_types::Geometry::MultiLineString(g_multilinestring),
            w_multilinestring.try_into().unwrap()
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
            w_multilinestring.try_into().unwrap()
        );
    }

    #[test]
    fn convert_empty_multipoint() {
        let w_multipoint = MultiPoint(vec![]).as_item();
        let g_multipoint: geo_types::MultiPoint<f64> = geo_types::MultiPoint(vec![]);
        assert_eq!(
            geo_types::Geometry::MultiPoint(g_multipoint),
            w_multipoint.try_into().unwrap()
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
            w_multipoint.try_into().unwrap()
        );
    }

    #[test]
    fn convert_empty_multipolygon() {
        let w_multipolygon = MultiPolygon(vec![]).as_item();
        let g_multipolygon: geo_types::MultiPolygon<f64> = geo_types::MultiPolygon(vec![]);
        assert_eq!(
            geo_types::Geometry::MultiPolygon(g_multipolygon),
            w_multipolygon.try_into().unwrap()
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
            w_multipolygon.try_into().unwrap()
        );
    }

    #[test]
    fn convert_empty_geometrycollection() {
        let w_geometrycollection = GeometryCollection(vec![]).as_item();
        let g_geometrycollection: geo_types::GeometryCollection<f64> =
            geo_types::GeometryCollection(vec![]);
        assert_eq!(
            geo_types::Geometry::GeometryCollection(g_geometrycollection),
            w_geometrycollection.try_into().unwrap()
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
            w_geometrycollection.try_into().unwrap()
        );
    }
}
