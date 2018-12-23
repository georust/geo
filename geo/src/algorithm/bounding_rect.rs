use geo_types::private_utils::{get_bounding_rect, line_string_bounding_rect};
use {
    Coordinate, CoordinateType, Line, LineString, MultiLineString, MultiPoint, MultiPolygon,
    Polygon, Rect, Triangle,
};

/// Calculation of the bounding rectangle of a geometry.
pub trait BoundingRect<T: CoordinateType> {
    type Output;

    /// Return the bounding rectangle of a geometry
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::{Point, LineString};
    /// use geo::algorithm::bounding_rect::BoundingRect;
    ///
    /// let mut vec = Vec::new();
    /// vec.push(Point::new(40.02f64, 116.34));
    /// vec.push(Point::new(42.02f64, 116.34));
    /// vec.push(Point::new(42.02f64, 118.34));
    /// let linestring = LineString::from(vec);
    /// let bounding_rect = linestring.bounding_rect().unwrap();
    ///
    /// assert_eq!(40.02f64, bounding_rect.min.x);
    /// assert_eq!(42.02f64, bounding_rect.max.x);
    /// assert_eq!(116.34, bounding_rect.min.y);
    /// assert_eq!(118.34, bounding_rect.max.y);
    /// ```
    ///
    fn bounding_rect(&self) -> Self::Output;
}

impl<T> BoundingRect<T> for MultiPoint<T>
where
    T: CoordinateType,
{
    type Output = Option<Rect<T>>;

    ///
    /// Return the BoundingRect for a MultiPoint
    ///
    fn bounding_rect(&self) -> Self::Output {
        get_bounding_rect(self.0.iter().map(|p| p.0))
    }
}

impl<T> BoundingRect<T> for Line<T>
where
    T: CoordinateType,
{
    type Output = Rect<T>;

    fn bounding_rect(&self) -> Self::Output {
        let a = self.start;
        let b = self.end;
        let (xmin, xmax) = if a.x <= b.x { (a.x, b.x) } else { (b.x, a.x) };
        let (ymin, ymax) = if a.y <= b.y { (a.y, b.y) } else { (b.y, a.y) };
        Rect {
            min: Coordinate { x: xmin, y: ymin },
            max: Coordinate { x: xmax, y: ymax },
        }
    }
}

impl<T> BoundingRect<T> for LineString<T>
where
    T: CoordinateType,
{
    type Output = Option<Rect<T>>;

    ///
    /// Return the BoundingRect for a LineString
    ///
    fn bounding_rect(&self) -> Self::Output {
        line_string_bounding_rect(self)
    }
}

impl<T> BoundingRect<T> for MultiLineString<T>
where
    T: CoordinateType,
{
    type Output = Option<Rect<T>>;

    ///
    /// Return the BoundingRect for a MultiLineString
    ///
    fn bounding_rect(&self) -> Self::Output {
        get_bounding_rect(self.0.iter().flat_map(|line| line.0.iter().map(|c| *c)))
    }
}

impl<T> BoundingRect<T> for Polygon<T>
where
    T: CoordinateType,
{
    type Output = Option<Rect<T>>;

    ///
    /// Return the BoundingRect for a Polygon
    ///
    fn bounding_rect(&self) -> Self::Output {
        let line = &self.exterior;
        get_bounding_rect(line.0.iter().cloned())
    }
}

impl<T> BoundingRect<T> for MultiPolygon<T>
where
    T: CoordinateType,
{
    type Output = Option<Rect<T>>;

    ///
    /// Return the BoundingRect for a MultiPolygon
    ///
    fn bounding_rect(&self) -> Self::Output {
        get_bounding_rect(
            self.0
                .iter()
                .flat_map(|poly| (poly.exterior).0.iter().map(|c| *c)),
        )
    }
}

impl<T> BoundingRect<T> for Triangle<T>
where
    T: CoordinateType,
{
    type Output = Rect<T>;

    fn bounding_rect(&self) -> Self::Output {
        get_bounding_rect(self.to_array().iter().map(|n| *n)).unwrap()
    }
}

#[cfg(test)]
mod test {
    use algorithm::bounding_rect::BoundingRect;
    use {
        Coordinate, Line, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon,
        Rect,
    };

    #[test]
    fn empty_linestring_test() {
        let linestring: LineString<f32> = LineString(vec![]);
        let bounding_rect = linestring.bounding_rect();
        assert!(bounding_rect.is_none());
    }
    #[test]
    fn linestring_one_point_test() {
        let vec = vec![(40.02f64, 116.34)];
        let linestring = LineString::from(vec);
        let bounding_rect = Rect {
            min: Coordinate {
                x: 40.02f64,
                y: 116.34,
            },
            max: Coordinate {
                x: 40.02,
                y: 116.34,
            },
        };
        assert_eq!(bounding_rect, linestring.bounding_rect().unwrap());
    }
    #[test]
    fn linestring_test() {
        let linestring = LineString::from(vec![(1., 1.), (2., -2.), (-3., -3.), (-4., 4.)]);
        let bounding_rect = Rect {
            min: Coordinate { x: -4., y: -3. },
            max: Coordinate { x: 2., y: 4. },
        };
        assert_eq!(bounding_rect, linestring.bounding_rect().unwrap());
    }
    #[test]
    fn multilinestring_test() {
        let multiline = MultiLineString(vec![
            LineString::from(vec![(1., 1.), (-40., 1.)]),
            LineString::from(vec![(1., 1.), (50., 1.)]),
            LineString::from(vec![(1., 1.), (1., -60.)]),
            LineString::from(vec![(1., 1.), (1., 70.)]),
        ]);
        let bounding_rect = Rect {
            min: Coordinate { x: -40., y: -60. },
            max: Coordinate { x: 50., y: 70. },
        };
        assert_eq!(bounding_rect, multiline.bounding_rect().unwrap());
    }
    #[test]
    fn multipoint_test() {
        let p = |x, y| Point(Coordinate { x, y });
        let multipoint = MultiPoint(vec![p(1., 1.), p(2., -2.), p(-3., -3.), p(-4., 4.)]);
        let bounding_rect = Rect {
            min: Coordinate { x: -4., y: -3. },
            max: Coordinate { x: 2., y: 4. },
        };
        assert_eq!(bounding_rect, multipoint.bounding_rect().unwrap());
    }
    #[test]
    fn polygon_test() {
        let linestring = LineString::from(vec![(0., 0.), (5., 0.), (5., 6.), (0., 6.), (0., 0.)]);
        let line_bounding_rect = linestring.bounding_rect().unwrap();
        let poly = Polygon::new(linestring, Vec::new());
        assert_eq!(line_bounding_rect, poly.bounding_rect().unwrap());
    }
    #[test]
    fn multipolygon_test() {
        let mpoly = MultiPolygon(vec![
            Polygon::new(
                LineString::from(vec![(0., 0.), (50., 0.), (0., -70.), (0., 0.)]),
                Vec::new(),
            ),
            Polygon::new(
                LineString::from(vec![(0., 0.), (5., 0.), (0., 80.), (0., 0.)]),
                Vec::new(),
            ),
            Polygon::new(
                LineString::from(vec![(0., 0.), (-60., 0.), (0., 6.), (0., 0.)]),
                Vec::new(),
            ),
        ]);
        let bounding_rect = Rect {
            min: Coordinate { x: -60., y: -70. },
            max: Coordinate { x: 50., y: 80. },
        };
        assert_eq!(bounding_rect, mpoly.bounding_rect().unwrap());
    }
    #[test]
    fn line_test() {
        let line1 = Line::new(Coordinate { x: 0., y: 1. }, Coordinate { x: 2., y: 3. });
        let line2 = Line::new(Coordinate { x: 2., y: 3. }, Coordinate { x: 0., y: 1. });
        assert_eq!(
            line1.bounding_rect(),
            Rect {
                min: Coordinate { x: 0., y: 1. },
                max: Coordinate { x: 2., y: 3. },
            }
        );
        assert_eq!(
            line2.bounding_rect(),
            Rect {
                min: Coordinate { x: 0., y: 1. },
                max: Coordinate { x: 2., y: 3. },
            }
        );
    }
}
