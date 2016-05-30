use types::Point;

/// Returns the distance between two geometries.

pub trait Distance<RHS = Self> {
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
    fn distance(&self, rhs: &RHS) -> f64;
}

impl Distance<Point> for Point {
    fn distance(&self, p: &Point) -> f64 {
        ((self.x() - p.x()).powi(2) + (self.y() - p.y()).powi(2)).sqrt()
    }
}

#[cfg(test)]
mod test {
    use types::Point;
    use algorithm::distance::Distance;
    #[test]
    fn distance1_test() {
        assert_eq!(Point::new(0., 0.).distance(&Point::new(1., 0.)), 1.);
    }
    #[test]
    fn distance2_test() {
        // Point::new(-72.1235, 42.3521).distance(&Point::new(72.1260, 70.612)) = 146.99163308930207
        let dist = Point::new(-72.1235, 42.3521).distance(&Point::new(72.1260, 70.612));
        assert!(dist < 147. && dist > 146.);
    }
}
