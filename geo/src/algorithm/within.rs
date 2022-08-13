use crate::algorithm::Contains;

/// Tests if a geometry is completely within another geometry.
///
/// In other words, the [DE-9IM] intersection matrix for (Self, Rhs) is `[T*F**F***]`
///
/// # Examples
///
/// ```
/// use geo::{point, line_string};
/// use geo::algorithm::Within;
///
/// let line_string = line_string![(x: 0.0, y: 0.0), (x: 2.0, y: 4.0)];
///
/// assert!(point!(x: 1.0, y: 2.0).is_within(&line_string));
///
/// // Note that a geometry on only the *boundary* of another geometry is not considered to
/// // be _within_ that geometry. See [`Relate`] for more information.
/// assert!(! point!(x: 0.0, y: 0.0).is_within(&line_string));
/// ```
///
/// `Within` is equivalent to [`Contains`] with the arguments swapped.
///
/// ```
/// use geo::{point, line_string};
/// use geo::algorithm::{Contains, Within};
///
/// let line_string = line_string![(x: 0.0, y: 0.0), (x: 2.0, y: 4.0)];
/// let point = point!(x: 1.0, y: 2.0);
///
/// // These two comparisons are completely equivalent
/// assert!(point.is_within(&line_string));
/// assert!(line_string.contains(&point));
/// ```
///
/// [DE-9IM]: https://en.wikipedia.org/wiki/DE-9IM
pub trait Within<Other> {
    fn is_within(&self, b: &Other) -> bool;
}

impl<G1, G2> Within<G2> for G1
where
    G2: Contains<G1>,
{
    fn is_within(&self, b: &G2) -> bool {
        b.contains(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{point, Rect};
    #[test]
    fn basic() {
        let a = point!(x: 1.0, y: 2.0);
        let b = Rect::new((0.0, 0.0), (3.0, 3.0)).to_polygon();
        assert!(a.is_within(&b));
    }
}
