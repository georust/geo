/// Calculate a minimum distance between two geometries in a way that is useful for sorting operations
pub trait ComparableDistance<F, Origin, Destination> {
    /// This trait differs from [Distance](geo::Distance) in that the value returned is not a true distance 
    /// but a value that is indicative of the true distance that can be used for sorting. It is generally
    /// faster to compute this value rather than true distance if all you need is a comparable figure between two 
    /// geometries 
    /// 
    /// Note that not all implementations support all geometry combinations, but at least `Point` to `Point`
    /// is supported.
    /// See [specific implementations](#implementors) for details.
    ///
    /// # Units
    ///
    /// - `origin`, `destination`: geometry where the units of x/y depend on the trait implementation.
    /// - returns: depends on the trait implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::{Euclidean, ComparableDistance, Point};
    /// let p1: Point = Point::new(0.0, 0.0);
    /// let p2: Point = Point::new(0.0, 2.0);
    /// let p3: Point = Point::new(0.0, 5.0);
    /// 
    /// let comparable_1_2 = Euclidean.comparable_distance(p1, p2);
    /// let comparable_1_3 = Euclidean.comparable_distance(p1, p3);
    ///
    /// assert_eq!(comparable_1_2, 4.0);
    /// assert_eq!(comparable_1_3, 25.0);
    /// assert_lt!(comparable_1_2, comparable_1_3);
    ///
    /// ```
    fn comparable_distance(&self, origin: Origin, destination: Destination) -> F;
}
