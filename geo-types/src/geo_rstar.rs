use crate::private_utils::{
    bounding_rect_merge, get_bounding_rect, line_bounding_rect, line_string_bounding_rect,
    point_line_euclidean_distance, point_line_string_euclidean_distance,
};
use crate::{
    Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPoint, MultiPolygon,
    Point, Polygon, Rect, Triangle,
};
use num_traits::Bounded;

impl<T> rstar::Point for Point<T>
where
    T: num_traits::Float + rstar::RTreeNum,
{
    type Scalar = T;

    const DIMENSIONS: usize = 2;

    fn generate(generator: impl Fn(usize) -> Self::Scalar) -> Self {
        Point::new(generator(0), generator(1))
    }

    fn nth(&self, index: usize) -> Self::Scalar {
        match index {
            0 => self.0.x,
            1 => self.0.y,
            _ => unreachable!(),
        }
    }
    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.0.x,
            1 => &mut self.0.y,
            _ => unreachable!(),
        }
    }
}

impl<T> rstar::RTreeObject for Line<T>
where
    T: num_traits::Float + rstar::RTreeNum,
{
    type Envelope = rstar::AABB<Point<T>>;

    fn envelope(&self) -> Self::Envelope {
        let bounding_rect = line_bounding_rect(*self);
        rstar::AABB::from_corners(bounding_rect.min().into(), bounding_rect.max().into())
    }
}

impl<T> rstar::RTreeObject for LineString<T>
where
    T: num_traits::Float + rstar::RTreeNum,
{
    type Envelope = rstar::AABB<Point<T>>;

    fn envelope(&self) -> Self::Envelope {
        let bounding_rect = line_string_bounding_rect(self);
        match bounding_rect {
            None => rstar::AABB::from_corners(
                Point::new(Bounded::min_value(), Bounded::min_value()),
                Point::new(Bounded::max_value(), Bounded::max_value()),
            ),
            Some(b) => rstar::AABB::from_corners(
                Point::new(b.min().x, b.min().y),
                Point::new(b.max().x, b.max().y),
            ),
        }
    }
}

impl<T> rstar::RTreeObject for Polygon<T>
where
    T: num_traits::Float + rstar::RTreeNum,
{
    type Envelope = rstar::AABB<Point<T>>;

    fn envelope(&self) -> Self::Envelope {
        self.exterior().envelope()
    }
}

impl<T> rstar::RTreeObject for MultiPoint<T>
where
    T: num_traits::Float + rstar::RTreeNum,
{
    type Envelope = rstar::AABB<Point<T>>;

    fn envelope(&self) -> Self::Envelope {
        unimplemented!()
    }
}

impl<T> rstar::RTreeObject for MultiLineString<T>
where
    T: num_traits::Float + rstar::RTreeNum,
{
    type Envelope = rstar::AABB<Point<T>>;

    fn envelope(&self) -> Self::Envelope {
        get_bounding_rect(
            self.0
                .iter()
                .flat_map(|poly| poly.exterior().0.iter().cloned()),
        )
    }
}

impl<T> rstar::RTreeObject for MultiPolygon<T>
where
    T: num_traits::Float + rstar::RTreeNum,
{
    type Envelope = rstar::AABB<Point<T>>;

    fn envelope(&self) -> Self::Envelope {
        unimplemented!()
    }
}

impl<T> rstar::RTreeObject for GeometryCollection<T>
where
    T: num_traits::Float + rstar::RTreeNum,
{
    type Envelope = rstar::AABB<Point<T>>;

    fn envelope(&self) -> Self::Envelope {
        unimplemented!()
    }
}

impl<T> rstar::RTreeObject for Rect<T>
where
    T: num_traits::Float + rstar::RTreeNum,
{
    type Envelope = rstar::AABB<Point<T>>;

    fn envelope(&self) -> Self::Envelope {
        unimplemented!()
    }
}

impl<T> rstar::RTreeObject for Triangle<T>
where
    T: num_traits::Float + rstar::RTreeNum,
{
    type Envelope = rstar::AABB<Point<T>>;

    fn envelope(&self) -> Self::Envelope {
        get_bounding_rect(self.to_array().iter().cloned()).unwrap()
    }
}

impl<T> rstar::RTreeObject for Geometry<T>
where
    T: num_traits::Float + rstar::RTreeNum,
{
    type Envelope = rstar::AABB<Point<T>>;

    fn envelope(&self) -> Self::Envelope {
        match self {
            Geometry::Point(g) => g.envelope(),
            Geometry::Line(g) => g.envelope(),
            Geometry::LineString(g) => g.envelope(),
            Geometry::Polygon(g) => g.envelope(),
            Geometry::MultiPoint(g) => g.envelope(),
            Geometry::MultiLineString(g) => g.envelope(),
            Geometry::MultiPolygon(g) => g.envelope(),
            Geometry::GeometryCollection(g) => g.envelope(),
            Geometry::Rect(g) => g.envelope(),
            Geometry::Triangle(g) => g.envelope(),
        }
    }
}

impl<T> rstar::PointDistance for Line<T>
where
    T: num_traits::Float + rstar::RTreeNum,
{
    fn distance_2(&self, point: &Point<T>) -> T {
        let d = point_line_euclidean_distance(*point, *self);
        d.powi(2)
    }
}

impl<T> rstar::PointDistance for LineString<T>
where
    T: num_traits::Float + rstar::RTreeNum,
{
    fn distance_2(&self, point: &Point<T>) -> T {
        let d = point_line_string_euclidean_distance(*point, self);
        if d == T::zero() {
            d
        } else {
            d.powi(2)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{Coordinate, Line, Point};

    #[test]
    /// ensure Line's SpatialObject impl is correct
    fn line_test() {
        use rstar::primitives::Line as RStarLine;
        use rstar::{PointDistance, RTreeObject};

        let rl = RStarLine::new(Point::new(0.0, 0.0), Point::new(5.0, 5.0));
        let l = Line::new(Coordinate { x: 0.0, y: 0.0 }, Coordinate { x: 5., y: 5. });
        assert_eq!(rl.envelope(), l.envelope());
        // difference in 15th decimal place
        assert_relative_eq!(26.0, rl.distance_2(&Point::new(4.0, 10.0)));
        assert_relative_eq!(25.999999999999996, l.distance_2(&Point::new(4.0, 10.0)));
    }
}
