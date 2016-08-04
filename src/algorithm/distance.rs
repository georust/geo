use num::Float;
use types::{Point};

/// Returns the distance between two geometries.

pub trait Distance<T, Rhs = Self>
{
    /// Returns the distance between two points:
    ///
    /// ```
    /// use geo::{COORD_PRECISION, Point};
    /// use geo::algorithm::distance::Distance;
    ///
    /// let p = Point::new(-72.1235, 42.3521);
    /// let dist = p.distance(&Point::new(-72.1260, 42.45));
    /// assert!(dist < COORD_PRECISION)
    /// ```
    fn distance(&self, rhs: &Rhs) -> T;
}

impl<T> Distance<T, Point<T>> for Point<T>
    where T: Float
{
    fn distance(&self, p: &Point<T>) -> T {
        let (dx, dy) = (self.x() - p.x(), self.y() - p.y());
        (dx * dx + dy * dy).sqrt()
    }
}

#[cfg(test)]
mod test {
    use types::{Point};
    use algorithm::distance::{Distance};
    #[test]
    fn distance1_test() {
        assert_eq!(Point::<f64>::new(0., 0.).distance(&Point::<f64>::new(1., 0.)), 1.);
    }
    #[test]
    fn distance2_test() {
        // Point::new(-72.1235, 42.3521).distance(&Point::new(72.1260, 70.612)) = 146.99163308930207
        let dist = Point::new(-72.1235, 42.3521).distance(&Point::new(72.1260, 70.612));
        assert!(dist < 147. && dist > 146.);
    }
}
