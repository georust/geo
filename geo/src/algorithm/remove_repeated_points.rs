use crate::{
    CoordNum, Geometry, Line, LineString, MultiLineString, MultiPoint, MultiPolygon, Point,
    Polygon, Rect, Triangle,
};
use geo_types::GeometryCollection;
use num_traits::FromPrimitive;

/// Remove repeated points from a `MultiPoint` and repeated consecutive coordinates
/// from `LineString`, `Polygon`, `MultiLineString` and `MultiPolygon`.
///
/// For `GeometryCollection` it individually removes the repeated points
/// of each geometry in the collection.
///
/// For `Point`, `Line`, `Rect` and `Triangle` it returns a clone of the geometry.
pub trait RemoveRepeatedPoints<T>
where
    T: CoordNum + FromPrimitive,
{
    /// Create a new geometry with (consecutive) repeated points removed.
    fn remove_repeated_points(&self) -> Self;
}

impl<T> RemoveRepeatedPoints<T> for MultiPoint<T>
where
    T: CoordNum + FromPrimitive,
{
    /// Create a MultiPoint with repeated points removed.
    fn remove_repeated_points(&self) -> Self {
        let mut points = vec![];
        for p in self.0.iter() {
            if !points.contains(p) {
                points.push(*p);
            }
        }
        MultiPoint(points)
    }
}

impl<T> RemoveRepeatedPoints<T> for LineString<T>
where
    T: CoordNum + FromPrimitive,
{
    /// Create a LineString with consecutive repeated points removed.
    fn remove_repeated_points(&self) -> Self {
        let mut coords = self.0.clone();
        coords.dedup();
        LineString(coords)
    }
}

impl<T> RemoveRepeatedPoints<T> for Polygon<T>
where
    T: CoordNum + FromPrimitive,
{
    /// Create a Polygon with consecutive repeated points removed.
    fn remove_repeated_points(&self) -> Self {
        Polygon::new(
            self.exterior().remove_repeated_points(),
            self.interiors()
                .iter()
                .map(|ls| ls.remove_repeated_points())
                .collect(),
        )
    }
}

impl<T> RemoveRepeatedPoints<T> for MultiLineString<T>
where
    T: CoordNum + FromPrimitive,
{
    /// Create a MultiLineString with consecutive repeated points removed.
    fn remove_repeated_points(&self) -> Self {
        MultiLineString::new(
            self.0
                .iter()
                .map(|ls| ls.remove_repeated_points())
                .collect(),
        )
    }
}

impl<T> RemoveRepeatedPoints<T> for MultiPolygon<T>
where
    T: CoordNum + FromPrimitive,
{
    /// Create a MultiPolygon with consecutive repeated points removed.
    fn remove_repeated_points(&self) -> Self {
        MultiPolygon::new(self.0.iter().map(|p| p.remove_repeated_points()).collect())
    }
}

// Implementation that returns a clone of the geometry for
// Point / Line / Triangle / Rect types (that are not candidate for coordinates removal)
macro_rules! return_self_impl {
    ($type:ident) => {
        impl<T> RemoveRepeatedPoints<T> for $type<T>
        where
            T: CoordNum + FromPrimitive,
        {
            fn remove_repeated_points(&self) -> Self {
                self.clone()
            }
        }
    };
}

return_self_impl!(Point);
return_self_impl!(Rect);
return_self_impl!(Triangle);
return_self_impl!(Line);

impl<T> RemoveRepeatedPoints<T> for GeometryCollection<T>
where
    T: CoordNum + FromPrimitive,
{
    /// Create a GeometryCollection with (consecutive) repeated points
    /// of its geometries removed.
    fn remove_repeated_points(&self) -> Self {
        GeometryCollection::new_from(self.0.iter().map(|g| g.remove_repeated_points()).collect())
    }
}

impl<T> RemoveRepeatedPoints<T> for Geometry<T>
where
    T: CoordNum + FromPrimitive,
{
    /// Create a Geometry with consecutive repeated points removed.
    fn remove_repeated_points(&self) -> Self {
        match self {
            Geometry::Point(p) => Geometry::Point(p.remove_repeated_points()),
            Geometry::Line(l) => Geometry::Line(l.remove_repeated_points()),
            Geometry::LineString(ls) => Geometry::LineString(ls.remove_repeated_points()),
            Geometry::Polygon(p) => Geometry::Polygon(p.remove_repeated_points()),
            Geometry::MultiPoint(mp) => Geometry::MultiPoint(mp.remove_repeated_points()),
            Geometry::MultiLineString(mls) => {
                Geometry::MultiLineString(mls.remove_repeated_points())
            }
            Geometry::MultiPolygon(mp) => Geometry::MultiPolygon(mp.remove_repeated_points()),
            Geometry::Rect(r) => Geometry::Rect(r.remove_repeated_points()),
            Geometry::Triangle(t) => Geometry::Triangle(t.remove_repeated_points()),
            Geometry::GeometryCollection(gc) => {
                Geometry::GeometryCollection(gc.remove_repeated_points())
            }
        }
    }
}

// The following can't be used until
// "impl<T: CoordNum> From<GeometryCollection<T>> for Geometry<T>" is implemented
// (see geo-types/src/geometry/mod.rs, lines 101-106)
//
// impl<T> RemoveRepeatedPoints<T> for Geometry<T>
// where
//     T: CoordNum + FromPrimitive,
// {
//     crate::geometry_delegate_impl! {
//         fn remove_repeated_points(&self) -> Geometry<T>;
//     }
// }

#[cfg(test)]
mod test {
    use crate::RemoveRepeatedPoints;
    use crate::{
        Coord, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon, Point,
        Polygon,
    };

    #[test]
    fn test_remove_repeated_points_multipoint_integer() {
        let mp = MultiPoint(vec![
            Point::new(0, 0),
            Point::new(1, 1),
            Point::new(1, 1),
            Point::new(1, 1),
            Point::new(2, 2),
            Point::new(0, 0),
        ]);

        let expected = MultiPoint(vec![Point::new(0, 0), Point::new(1, 1), Point::new(2, 2)]);

        assert_eq!(mp.remove_repeated_points(), expected);
    }

    #[test]
    fn test_remove_repeated_points_multipoint() {
        let mp = MultiPoint(vec![
            Point::new(0., 0.),
            Point::new(1., 1.),
            Point::new(1., 1.),
            Point::new(1., 1.),
            Point::new(2., 2.),
            Point::new(0., 0.),
        ]);

        let expected = MultiPoint(vec![
            Point::new(0., 0.),
            Point::new(1., 1.),
            Point::new(2., 2.),
        ]);

        assert_eq!(mp.remove_repeated_points(), expected);
    }

    #[test]
    fn test_remove_repeated_points_linestring() {
        let ls = LineString(vec![
            Coord { x: 0., y: 0. },
            Coord { x: 1., y: 1. },
            Coord { x: 1., y: 1. },
            Coord { x: 1., y: 1. },
            Coord { x: 2., y: 2. },
            Coord { x: 2., y: 2. },
            Coord { x: 0., y: 0. },
        ]);

        let expected = LineString(vec![
            Coord { x: 0., y: 0. },
            Coord { x: 1., y: 1. },
            Coord { x: 2., y: 2. },
            Coord { x: 0., y: 0. },
        ]);

        assert_eq!(ls.remove_repeated_points(), expected);
    }

    #[test]
    fn test_remove_repeated_points_polygon() {
        let poly = Polygon::new(
            LineString(vec![
                Coord { x: 0., y: 0. },
                Coord { x: 1., y: 1. },
                Coord { x: 1., y: 1. },
                Coord { x: 1., y: 1. },
                Coord { x: 0., y: 2. },
                Coord { x: 0., y: 2. },
                Coord { x: 0., y: 0. },
            ]),
            vec![],
        );

        let expected = Polygon::new(
            LineString(vec![
                Coord { x: 0., y: 0. },
                Coord { x: 1., y: 1. },
                Coord { x: 0., y: 2. },
                Coord { x: 0., y: 0. },
            ]),
            vec![],
        );

        assert_eq!(poly.remove_repeated_points(), expected);
    }

    #[test]
    fn test_remove_repeated_points_multilinestring() {
        let mls = MultiLineString(vec![
            LineString(vec![
                Coord { x: 0., y: 0. },
                Coord { x: 1., y: 1. },
                Coord { x: 1., y: 1. },
                Coord { x: 1., y: 1. },
                Coord { x: 2., y: 2. },
                Coord { x: 2., y: 2. },
                Coord { x: 0., y: 0. },
            ]),
            LineString(vec![
                Coord { x: 10., y: 10. },
                Coord { x: 11., y: 11. },
                Coord { x: 11., y: 11. },
                Coord { x: 11., y: 11. },
                Coord { x: 12., y: 12. },
                Coord { x: 12., y: 12. },
                Coord { x: 10., y: 10. },
            ]),
        ]);

        let expected = MultiLineString(vec![
            LineString(vec![
                Coord { x: 0., y: 0. },
                Coord { x: 1., y: 1. },
                Coord { x: 2., y: 2. },
                Coord { x: 0., y: 0. },
            ]),
            LineString(vec![
                Coord { x: 10., y: 10. },
                Coord { x: 11., y: 11. },
                Coord { x: 12., y: 12. },
                Coord { x: 10., y: 10. },
            ]),
        ]);

        assert_eq!(mls.remove_repeated_points(), expected);
    }

    #[test]
    fn test_remove_repeated_points_multipolygon() {
        let mpoly = MultiPolygon(vec![
            Polygon::new(
                LineString(vec![
                    Coord { x: 0., y: 0. },
                    Coord { x: 1., y: 1. },
                    Coord { x: 1., y: 1. },
                    Coord { x: 1., y: 1. },
                    Coord { x: 0., y: 2. },
                    Coord { x: 0., y: 2. },
                    Coord { x: 0., y: 0. },
                ]),
                vec![],
            ),
            Polygon::new(
                LineString(vec![
                    Coord { x: 10., y: 10. },
                    Coord { x: 11., y: 11. },
                    Coord { x: 11., y: 11. },
                    Coord { x: 11., y: 11. },
                    Coord { x: 10., y: 12. },
                    Coord { x: 10., y: 12. },
                    Coord { x: 10., y: 10. },
                ]),
                vec![],
            ),
        ]);

        let expected = MultiPolygon(vec![
            Polygon::new(
                LineString(vec![
                    Coord { x: 0., y: 0. },
                    Coord { x: 1., y: 1. },
                    Coord { x: 0., y: 2. },
                    Coord { x: 0., y: 0. },
                ]),
                vec![],
            ),
            Polygon::new(
                LineString(vec![
                    Coord { x: 10., y: 10. },
                    Coord { x: 11., y: 11. },
                    Coord { x: 10., y: 12. },
                    Coord { x: 10., y: 10. },
                ]),
                vec![],
            ),
        ]);

        assert_eq!(mpoly.remove_repeated_points(), expected);
    }

    #[test]
    fn test_remove_repeated_points_geometrycollection() {
        let gc = GeometryCollection::new_from(vec![
            MultiPoint(vec![
                Point::new(0., 0.),
                Point::new(1., 1.),
                Point::new(1., 1.),
                Point::new(1., 1.),
                Point::new(2., 2.),
                Point::new(0., 0.),
            ])
            .into(),
            LineString(vec![
                Coord { x: 0., y: 0. },
                Coord { x: 1., y: 1. },
                Coord { x: 1., y: 1. },
                Coord { x: 1., y: 1. },
                Coord { x: 2., y: 2. },
                Coord { x: 2., y: 2. },
                Coord { x: 0., y: 0. },
            ])
            .into(),
        ]);

        let expected = GeometryCollection::new_from(vec![
            MultiPoint(vec![
                Point::new(0., 0.),
                Point::new(1., 1.),
                Point::new(2., 2.),
            ])
            .into(),
            LineString(vec![
                Coord { x: 0., y: 0. },
                Coord { x: 1., y: 1. },
                Coord { x: 2., y: 2. },
                Coord { x: 0., y: 0. },
            ])
            .into(),
        ]);

        assert_eq!(gc.remove_repeated_points(), expected);
    }
}
