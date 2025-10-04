/// Calculate the minimum distance between two geometries.
pub trait Distance<F, Origin, Destination> {
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
    /// use geo::{Haversine, Euclidean, Distance, Point};
    /// let p1: Point = Point::new(0.0, 0.0);
    /// let p2: Point = Point::new(0.0, 2.0);
    ///
    /// assert_eq!(Euclidean.distance(p1, p2), 2.0);
    ///
    /// // The units of the output depend on the metric space.
    /// // In the case of [`Haversine`], it's meters.
    /// // See the documentation for each metric space for details.
    /// assert_eq!(Haversine.distance(p1, p2).round(), 222_390.0);
    /// ```
    fn distance(&self, origin: Origin, destination: Destination) -> F;

    /// Returns `true` if the minimum distance between `origin` and `destination` is less than
    /// or equal to `distance`
    ///
    /// # Notes
    /// When the calculated distance is 0 (intersecting / touching / overlapping geometries), _any_ positive `distance` value will return `True`
    fn distance_within(&self, origin: Origin, destination: Destination, distance: F) -> bool
    where
        F: PartialOrd,
    {
        self.distance(origin, destination) <= distance
    }
}
