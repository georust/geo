use {
    Bbox, Coordinate, CoordinateType, Line, LineString, MultiLineString, MultiPoint, MultiPolygon,
    Polygon,
};

/// Calculation of the bounding box of a geometry.

pub trait BoundingBox<T: CoordinateType> {
    type Output;

    /// Return the Bounding Box of a geometry
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::{Point, LineString};
    /// use geo::algorithm::boundingbox::BoundingBox;
    ///
    /// let mut vec = Vec::new();
    /// vec.push(Point::new(40.02f64, 116.34));
    /// vec.push(Point::new(42.02f64, 116.34));
    /// vec.push(Point::new(42.02f64, 118.34));
    /// let linestring = LineString::from(vec);
    /// let bbox = linestring.bbox().unwrap();
    ///
    /// assert_eq!(40.02f64, bbox.xmin);
    /// assert_eq!(42.02f64, bbox.xmax);
    /// assert_eq!(116.34, bbox.ymin);
    /// assert_eq!(118.34, bbox.ymax);
    /// ```
    ///
    fn bbox(&self) -> Self::Output;
}

fn get_min_max<T>(p: T, min: T, max: T) -> (T, T)
where
    T: CoordinateType,
{
    if p > max {
        (min, p)
    } else if p < min {
        (p, max)
    } else {
        (min, max)
    }
}

fn get_bbox<I, T>(collection: I) -> Option<Bbox<T>>
where
    T: CoordinateType,
    I: IntoIterator<Item = Coordinate<T>>,
{
    let mut iter = collection.into_iter();
    if let Some(pnt) = iter.next() {
        let mut xrange = (pnt.x, pnt.x);
        let mut yrange = (pnt.y, pnt.y);
        for pnt in iter {
            let (px, py) = pnt.x_y();
            xrange = get_min_max(px, xrange.0, xrange.1);
            yrange = get_min_max(py, yrange.0, yrange.1);
        }
        return Some(Bbox {
            xmin: xrange.0,
            xmax: xrange.1,
            ymin: yrange.0,
            ymax: yrange.1,
        });
    }
    None
}

impl<T> BoundingBox<T> for MultiPoint<T>
where
    T: CoordinateType,
{
    type Output = Option<Bbox<T>>;

    ///
    /// Return the BoundingBox for a MultiPoint
    ///
    fn bbox(&self) -> Self::Output {
        get_bbox(self.0.iter().map(|p| p.0))
    }
}

impl<T> BoundingBox<T> for Line<T>
where
    T: CoordinateType,
{
    type Output = Bbox<T>;

    fn bbox(&self) -> Self::Output {
        let a = self.start;
        let b = self.end;
        let (xmin, xmax) = if a.x <= b.x { (a.x, b.x) } else { (b.x, a.x) };
        let (ymin, ymax) = if a.y <= b.y { (a.y, b.y) } else { (b.y, a.y) };
        Bbox {
            xmin,
            xmax,
            ymin,
            ymax,
        }
    }
}

impl<T> BoundingBox<T> for LineString<T>
where
    T: CoordinateType,
{
    type Output = Option<Bbox<T>>;

    ///
    /// Return the BoundingBox for a LineString
    ///
    fn bbox(&self) -> Self::Output {
        get_bbox(self.0.iter().cloned())
    }
}

impl<T> BoundingBox<T> for MultiLineString<T>
where
    T: CoordinateType,
{
    type Output = Option<Bbox<T>>;

    ///
    /// Return the BoundingBox for a MultiLineString
    ///
    fn bbox(&self) -> Self::Output {
        get_bbox(self.0.iter().flat_map(|line| line.0.iter().map(|c| *c)))
    }
}

impl<T> BoundingBox<T> for Polygon<T>
where
    T: CoordinateType,
{
    type Output = Option<Bbox<T>>;

    ///
    /// Return the BoundingBox for a Polygon
    ///
    fn bbox(&self) -> Self::Output {
        let line = &self.exterior;
        get_bbox(line.0.iter().cloned())
    }
}

impl<T> BoundingBox<T> for MultiPolygon<T>
where
    T: CoordinateType,
{
    type Output = Option<Bbox<T>>;

    ///
    /// Return the BoundingBox for a MultiPolygon
    ///
    fn bbox(&self) -> Self::Output {
        get_bbox(
            self.0
                .iter()
                .flat_map(|poly| (poly.exterior).0.iter().map(|c| *c)),
        )
    }
}

#[cfg(test)]
mod test {
    use algorithm::boundingbox::BoundingBox;
    use {
        Bbox, Coordinate, Line, LineString, MultiLineString, MultiPoint, MultiPolygon, Point,
        Polygon,
    };

    #[test]
    fn empty_linestring_test() {
        let linestring: LineString<f32> = LineString(vec![]);
        let bbox = linestring.bbox();
        assert!(bbox.is_none());
    }
    #[test]
    fn linestring_one_point_test() {
        let vec = vec![(40.02f64, 116.34)];
        let linestring = LineString::from(vec);
        let bbox = Bbox {
            xmin: 40.02f64,
            ymax: 116.34,
            xmax: 40.02,
            ymin: 116.34,
        };
        assert_eq!(bbox, linestring.bbox().unwrap());
    }
    #[test]
    fn linestring_test() {
        let linestring = LineString::from(vec![(1., 1.), (2., -2.), (-3., -3.), (-4., 4.)]);
        let bbox = Bbox {
            xmin: -4.,
            ymax: 4.,
            xmax: 2.,
            ymin: -3.,
        };
        assert_eq!(bbox, linestring.bbox().unwrap());
    }
    #[test]
    fn multilinestring_test() {
        let multiline = MultiLineString(vec![
            LineString::from(vec![(1., 1.), (-40., 1.)]),
            LineString::from(vec![(1., 1.), (50., 1.)]),
            LineString::from(vec![(1., 1.), (1., -60.)]),
            LineString::from(vec![(1., 1.), (1., 70.)]),
        ]);
        let bbox = Bbox {
            xmin: -40.,
            ymax: 70.,
            xmax: 50.,
            ymin: -60.,
        };
        assert_eq!(bbox, multiline.bbox().unwrap());
    }
    #[test]
    fn multipoint_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let multipoint = MultiPoint(vec![p(1., 1.), p(2., -2.), p(-3., -3.), p(-4., 4.)]);
        let bbox = Bbox {
            xmin: -4.,
            ymax: 4.,
            xmax: 2.,
            ymin: -3.,
        };
        assert_eq!(bbox, multipoint.bbox().unwrap());
    }
    #[test]
    fn polygon_test() {
        let linestring = LineString::from(vec![(0., 0.), (5., 0.), (5., 6.), (0., 6.), (0., 0.)]);
        let line_bbox = linestring.bbox().unwrap();
        let poly = Polygon::new(linestring, Vec::new());
        assert_eq!(line_bbox, poly.bbox().unwrap());
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
        let bbox = Bbox {
            xmin: -60.,
            ymax: 80.,
            xmax: 50.,
            ymin: -70.,
        };
        assert_eq!(bbox, mpoly.bbox().unwrap());
    }
    #[test]
    fn line_test() {
        let line1 = Line::new(Coordinate { x: 0., y: 1. }, Coordinate { x: 2., y: 3. });
        let line2 = Line::new(Coordinate { x: 2., y: 3. }, Coordinate { x: 0., y: 1. });
        assert_eq!(
            line1.bbox(),
            Bbox {
                xmin: 0.,
                xmax: 2.,
                ymin: 1.,
                ymax: 3.,
            }
        );
        assert_eq!(
            line2.bbox(),
            Bbox {
                xmin: 0.,
                xmax: 2.,
                ymin: 1.,
                ymax: 3.,
            }
        );
    }
}
