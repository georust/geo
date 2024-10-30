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
    fn distance(origin: Origin, destination: Destination) -> F;
}

// Distance is a symmetric operation, so we can implement it once for both
macro_rules! symmetric_distance_impl {
    ($a:ty, $b:ty, for: $metric_space: ty, where: $($bound:tt)+) => {
        impl<F> $crate::Distance<F, $a, $b> for $metric_space
        where
            F: $($bound)+,
        {
            fn distance(a: $a, b: $b) -> F {
                Self::distance(b, a)
            }
        }
    };
}
pub(crate) use symmetric_distance_impl;
