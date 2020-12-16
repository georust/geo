use crate::utils::{partial_max, partial_min};
use crate::{
    Coordinate, CoordinateType, Geometry, GeometryCollection, Line, LineString, MultiLineString,
    MultiPoint, MultiPolygon, Point, Polygon, Rect, Triangle,
};
use geo_types::private_utils::{get_bounding_rect, line_string_bounding_rect};

/// Calculation of the bounding rectangle of a geometry.
pub trait BoundingRect<T: CoordinateType> {
    /// Return the bounding rectangle of a geometry
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::algorithm::bounding_rect::BoundingRect;
    /// use geo::line_string;
    ///
    /// let line_string = line_string![
    ///     (x: 40.02f64, y: 116.34),
    ///     (x: 42.02f64, y: 116.34),
    ///     (x: 42.02f64, y: 118.34),
    /// ];
    ///
    /// let bounding_rect = line_string.bounding_rect().unwrap();
    ///
    /// assert_eq!(40.02f64, bounding_rect.min().x);
    /// assert_eq!(42.02f64, bounding_rect.max().x);
    /// assert_eq!(116.34, bounding_rect.min().y);
    /// assert_eq!(118.34, bounding_rect.max().y);
    /// ```
    fn bounding_rect(&self) -> Result<Rect<T>, ()>;
}

impl<T> BoundingRect<T> for Point<T>
where
    T: CoordinateType,
{
    /// Return the bounding rectangle for a `Point`. It will have zero width
    /// and zero height.
    fn bounding_rect(&self) -> Result<Rect<T>, ()> {
        Ok(Rect::new(self.0, self.0))
    }
}

impl<T> BoundingRect<T> for MultiPoint<T>
where
    T: CoordinateType,
{
    ///
    /// Return the BoundingRect for a MultiPoint
    fn bounding_rect(&self) -> Result<Rect<T>, ()> {
        get_bounding_rect(self.0.iter().map(|p| p.0)).ok_or(())
    }
}

impl<T> BoundingRect<T> for Line<T>
where
    T: CoordinateType,
{
    fn bounding_rect(&self) -> Result<Rect<T>, ()> {
        let a = self.start;
        let b = self.end;
        let (xmin, xmax) = if a.x <= b.x { (a.x, b.x) } else { (b.x, a.x) };
        let (ymin, ymax) = if a.y <= b.y { (a.y, b.y) } else { (b.y, a.y) };
        Ok(Rect::new(
            Coordinate { x: xmin, y: ymin },
            Coordinate { x: xmax, y: ymax },
        ))
    }
}

impl<T> BoundingRect<T> for LineString<T>
where
    T: CoordinateType,
{
    ///
    /// Return the BoundingRect for a LineString
    fn bounding_rect(&self) -> Result<Rect<T>, ()> {
        line_string_bounding_rect(self).ok_or(())
    }
}

impl<T> BoundingRect<T> for MultiLineString<T>
where
    T: CoordinateType,
{
    ///
    /// Return the BoundingRect for a MultiLineString
    fn bounding_rect(&self) -> Result<Rect<T>, ()> {
        get_bounding_rect(self.iter().flat_map(|line| line.0.iter().cloned())).ok_or(())
    }
}

impl<T> BoundingRect<T> for Polygon<T>
where
    T: CoordinateType,
{
    ///
    /// Return the BoundingRect for a Polygon
    fn bounding_rect(&self) -> Result<Rect<T>, ()> {
        let line = self.exterior();
        get_bounding_rect(line.0.iter().cloned()).ok_or(())
    }
}

impl<T> BoundingRect<T> for MultiPolygon<T>
where
    T: CoordinateType,
{
    ///
    /// Return the BoundingRect for a MultiPolygon
    fn bounding_rect(&self) -> Result<Rect<T>, ()> {
        get_bounding_rect(
            self.iter()
                .flat_map(|poly| poly.exterior().0.iter().cloned()),
        )
        .ok_or(())
    }
}

impl<T> BoundingRect<T> for Triangle<T>
where
    T: CoordinateType,
{
    fn bounding_rect(&self) -> Result<Rect<T>, ()> {
        get_bounding_rect(self.to_array().iter().cloned()).ok_or(())
    }
}

impl<T> BoundingRect<T> for Rect<T>
where
    T: CoordinateType,
{
    fn bounding_rect(&self) -> Result<Rect<T>, ()> {
        Ok(*self)
    }
}

impl<T> BoundingRect<T> for Geometry<T>
where
    T: CoordinateType,
{
    fn bounding_rect(&self) -> Result<Rect<T>, ()> {
        match self {
            Geometry::Point(g) => g.bounding_rect(),
            Geometry::Line(g) => g.bounding_rect(),
            Geometry::LineString(g) => g.bounding_rect(),
            Geometry::Polygon(g) => g.bounding_rect(),
            Geometry::MultiPoint(g) => g.bounding_rect(),
            Geometry::MultiLineString(g) => g.bounding_rect(),
            Geometry::MultiPolygon(g) => g.bounding_rect(),
            Geometry::GeometryCollection(g) => g.bounding_rect(),
            Geometry::Rect(g) => g.bounding_rect(),
            Geometry::Triangle(g) => g.bounding_rect(),
        }
    }
}

impl<T> BoundingRect<T> for GeometryCollection<T>
where
    T: CoordinateType,
{
    fn bounding_rect(&self) -> Result<Rect<T>, ()> {
        self.iter().fold(Err(()), |acc, next| {
            let next_bounding_rect = next.bounding_rect();

            match (acc, next_bounding_rect) {
                (Err(()), Err(())) => Err(()),
                (Ok(r), Err(())) | (Err(()), Ok(r)) => Ok(r),
                (Ok(r1), Ok(r2)) => Ok(bounding_rect_merge(r1, r2)),
            }
        })
    }
}

// Return a new rectangle that encompasses the provided rectangles
fn bounding_rect_merge<T: CoordinateType>(a: Rect<T>, b: Rect<T>) -> Rect<T> {
    Rect::new(
        Coordinate {
            x: partial_min(a.min().x, b.min().x),
            y: partial_min(a.min().y, b.min().y),
        },
        Coordinate {
            x: partial_max(a.max().x, b.max().x),
            y: partial_max(a.max().y, b.max().y),
        },
    )
}

#[cfg(test)]
mod test {
    use super::bounding_rect_merge;
    use crate::algorithm::bounding_rect::BoundingRect;
    use crate::line_string;
    use crate::{
        polygon, Coordinate, Geometry, GeometryCollection, Line, LineString, MultiLineString,
        MultiPoint, MultiPolygon, Point, Polygon, Rect,
    };

    #[test]
    fn empty_linestring_test() {
        let linestring: LineString<f32> = line_string![];
        let bounding_rect = linestring.bounding_rect();
        assert!(bounding_rect.is_err());
    }
    #[test]
    fn linestring_one_point_test() {
        let linestring = line_string![(x: 40.02f64, y: 116.34)];
        let bounding_rect = Rect::new(
            Coordinate {
                x: 40.02f64,
                y: 116.34,
            },
            Coordinate {
                x: 40.02,
                y: 116.34,
            },
        );
        assert_eq!(bounding_rect, linestring.bounding_rect().unwrap());
    }
    #[test]
    fn linestring_test() {
        let linestring = line_string![
            (x: 1., y: 1.),
            (x: 2., y: -2.),
            (x: -3., y: -3.),
            (x: -4., y: 4.)
        ];
        let bounding_rect = Rect::new(Coordinate { x: -4., y: -3. }, Coordinate { x: 2., y: 4. });
        assert_eq!(bounding_rect, linestring.bounding_rect().unwrap());
    }
    #[test]
    fn multilinestring_test() {
        let multiline = MultiLineString(vec![
            line_string![(x: 1., y: 1.), (x: -40., y: 1.)],
            line_string![(x: 1., y: 1.), (x: 50., y: 1.)],
            line_string![(x: 1., y: 1.), (x: 1., y: -60.)],
            line_string![(x: 1., y: 1.), (x: 1., y: 70.)],
        ]);
        let bounding_rect = Rect::new(
            Coordinate { x: -40., y: -60. },
            Coordinate { x: 50., y: 70. },
        );
        assert_eq!(bounding_rect, multiline.bounding_rect().unwrap());
    }
    #[test]
    fn multipoint_test() {
        let multipoint = MultiPoint::from(vec![(1., 1.), (2., -2.), (-3., -3.), (-4., 4.)]);
        let bounding_rect = Rect::new(Coordinate { x: -4., y: -3. }, Coordinate { x: 2., y: 4. });
        assert_eq!(bounding_rect, multipoint.bounding_rect().unwrap());
    }
    #[test]
    fn polygon_test() {
        let linestring = line_string![
            (x: 0., y: 0.),
            (x: 5., y: 0.),
            (x: 5., y: 6.),
            (x: 0., y: 6.),
            (x: 0., y: 0.),
        ];
        let line_bounding_rect = linestring.bounding_rect().unwrap();
        let poly = Polygon::new(linestring, Vec::new());
        assert_eq!(line_bounding_rect, poly.bounding_rect().unwrap());
    }
    #[test]
    fn multipolygon_test() {
        let mpoly = MultiPolygon(vec![
            polygon![(x: 0., y: 0.), (x: 50., y: 0.), (x: 0., y: -70.), (x: 0., y: 0.)],
            polygon![(x: 0., y: 0.), (x: 5., y: 0.), (x: 0., y: 80.), (x: 0., y: 0.)],
            polygon![(x: 0., y: 0.), (x: -60., y: 0.), (x: 0., y: 6.), (x: 0., y: 0.)],
        ]);
        let bounding_rect = Rect::new(
            Coordinate { x: -60., y: -70. },
            Coordinate { x: 50., y: 80. },
        );
        assert_eq!(bounding_rect, mpoly.bounding_rect().unwrap());
    }
    #[test]
    fn line_test() {
        let line1 = Line::new(Coordinate { x: 0., y: 1. }, Coordinate { x: 2., y: 3. });
        let line2 = Line::new(Coordinate { x: 2., y: 3. }, Coordinate { x: 0., y: 1. });
        assert_eq!(
            line1.bounding_rect().unwrap(),
            Rect::new(Coordinate { x: 0., y: 1. }, Coordinate { x: 2., y: 3. },)
        );
        assert_eq!(
            line2.bounding_rect().unwrap(),
            Rect::new(Coordinate { x: 0., y: 1. }, Coordinate { x: 2., y: 3. },)
        );
    }

    #[test]
    fn bounding_rect_merge_test() {
        assert_eq!(
            bounding_rect_merge(
                Rect::new(Coordinate { x: 0., y: 0. }, Coordinate { x: 1., y: 1. }),
                Rect::new(Coordinate { x: 1., y: 1. }, Coordinate { x: 2., y: 2. }),
            ),
            Rect::new(Coordinate { x: 0., y: 0. }, Coordinate { x: 2., y: 2. }),
        );
    }

    #[test]
    fn point_bounding_rect_test() {
        assert_eq!(
            Rect::new(Coordinate { x: 1., y: 2. }, Coordinate { x: 1., y: 2. }),
            Point(Coordinate { x: 1., y: 2. }).bounding_rect().unwrap(),
        );
    }

    #[test]
    fn geometry_collection_bounding_rect_test() {
        assert_eq!(
            Ok(Rect::new(
                Coordinate { x: 0., y: 0. },
                Coordinate { x: 1., y: 2. }
            )),
            GeometryCollection(vec![
                Geometry::Point(Point(Coordinate { x: 0., y: 0. })),
                Geometry::Point(Point(Coordinate { x: 1., y: 2. })),
            ])
            .bounding_rect(),
        );
    }
}
