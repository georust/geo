//! This module defines the [Vector2DOps] trait and implements it for the
//! [Coord] struct.
//!
//! In the future [Vector2DOps] might be implemented on other types:
//!
//! - Based on community discussions it seems like the existing struct
//!   [crate::Coord] is just one of many future data structures which may hold
//!   coordinate information. For example future data structures may also hold Z
//!   and/or M ordinates, or other arbitrary data.
//! - [geotraits::CoordTrait] is a future trait defining accessor methods
//!   `.x()` and `.y()` which will facilitate these generic data structures.
//!
//! > Note: [Vector2DOps] is not implemented for [crate::Point] because users
//! > probably expect to see more familiar geospatial functions like
//! > `a.euclidean_distance(b)` at that level and generally not linear algebra
//! > like `a.dot(b)`.
//!
//! For now, it is assumed that users of [Vector2DOps] are satisfied to upcast
//! everything to [CoordFloat]. In future it might make sense to split this
//! trait into 3 flavors supporting progressively more functions:
//!
//! - `trait Vector2DOpsCoordNum` for [CoordNum]
//!     - Supports magnitude_squared, dot
//! - `trait Vector2DOpsCoordNumSigned:Vector2DOpsCoordNum` for [CoordNum] + [Signed]
//!     - Supports left, right, wedge_product
//! - `trait Vector2DOpsCoordFloat:Vector2DOpsCoordNumSigned` for [CoordFloat]
//!     - Supports magnitude, normalize, etc
//!
//! Maybe if these traits were restricted to the future [geotraits::CoordTrait]
//! then they could provide default implementations using the accessors `.x()`
//! and `.y()`?
//!

use crate::{Coord, CoordFloat, CoordNum};

/// Defines vector operations for 2D coordinate types which implement CoordFloat
///
/// This trait is intended for internal use within the geo crate as a way to
/// bring together the various hand-crafted linear algebra operations used
/// throughout other algorithms and attached to various structs.
///
///

pub trait Vector2DOps<Rhs = Self>
where
    Self: Sized,
{
    type NumericType: CoordNum + Send + Sync;

    /// The euclidean distance between this coordinate and the origin
    ///
    /// `sqrt(x² + y²)`
    ///
    fn magnitude(self) -> Self::NumericType;

    /// The squared distance between this coordinate and the origin.
    /// (Avoids the square root calculation when it is not needed)
    ///
    /// `x² + y²`
    ///
    fn magnitude_squared(self) -> Self::NumericType;

    /// Rotate this coordinate around the origin by 90 degrees clockwise.
    ///
    /// `a.left() => (-a.y, a.x)`
    ///
    /// Assumes a coordinate system where positive `y` is up and positive `x` is
    /// to the right. The described rotation direction is consistent with the
    /// documentation for [crate::algorithm::rotate::Rotate].
    fn left(self) -> Self;

    /// Rotate this coordinate around the origin by 90 degrees anti-clockwise.
    ///
    /// `a.right() => (a.y, -a.x)`
    ///
    /// Assumes a coordinate system where positive `y` is up and positive `x` is
    /// to the right. The described rotation direction is consistent with the
    /// documentation for [crate::algorithm::rotate::Rotate].
    fn right(self) -> Self;

    /// The inner product of the coordinate components
    ///
    /// `a · b = a.x * b.x + a.y * b.y`
    ///
    fn dot_product(self, other: Rhs) -> Self::NumericType;

    /// The calculates the `wedge product` between two vectors.
    ///
    /// `a ∧ b = a.x * b.y - a.y * b.x`
    ///
    /// Also known as:
    ///
    ///  - `exterior product`
    ///    - because the wedge product comes from 'Exterior Algebra'
    ///  - `perpendicular product`
    ///    -  because it is equivalent to `a.dot(b.right())`
    ///  - `2D cross product`
    ///    - because it is equivalent to the signed magnitude of the
    ///      conventional 3D cross product assuming `z` ordinates are zero
    ///  - `determinant`
    ///    - because it is equivalent to the `determinant` of the 2x2 matrix
    ///      formed by the column-vector inputs.
    ///
    /// ## Examples
    ///
    /// The following list highlights some examples in geo which might be
    /// brought together to use this function:
    ///
    /// 1. [geo_types::Point::cross_prod()] is already defined on
    ///    [geo_types::Point]... but that it seems to be some other
    ///    operation on 3 points??
    /// 2. [geo_types::Line] struct also has a [geo_types::Line::determinant()]
    ///    function which is the same as `line.start.wedge_product(line.end)`
    /// 3. The [crate::algorithm::Kernel::orient2d()] trait default
    ///    implementation uses cross product to compute orientation. It returns
    ///    an enum, not the numeric value which is needed for line segment
    ///    intersection.
    ///
    /// ## Properties
    ///
    /// - The absolute value of the cross product is the area of the
    ///   parallelogram formed by the operands
    /// - Anti-commutative: The sign of the output is reversed if the operands
    ///   are reversed
    /// - If the operands are colinear with the origin, the value is zero
    /// - The sign can be used to check if the operands are clockwise with
    ///   respect to the origin, or phrased differently:
    ///   "is a to the left of the line between the origin and b"?
    ///   - If this is what you are using it for, then please use
    ///     [crate::algorithm::Kernel::orient2d()] instead as this is more
    ///     explicit and has a `RobustKernel` option for extra precision.
    fn wedge_product(self, other: Rhs) -> Self::NumericType;

    /// Try to find a vector of unit length in the same direction as this
    /// vector.
    ///
    /// Returns `None` if the magnitude of this vector is less than
    /// `minimum_magnitude` or the magnitude is not finite.
    ///  - For f32 the minimum_magnitude can be set to about 1e-30f32.
    ///  - For F64 the minimum_magnitude can be set to about 2e-301f64.
    ///
    /// These values should avoid overflowing to Infinity for coordinate values
    /// in the range typically relevant for spatial data (+-40e6) which is the
    /// approximate length of the earth's great circle in metres.
    ///
    /// > Note to Reviewer: it may be annoying to have to provide a value for
    /// > minimum_magnitude, but that seems to be what `nalgebra` does
    /// > (See https://docs.rs/nalgebra/latest/src/nalgebra/base/norm.rs.html#301-307).
    /// > Some other parts of the api do not require the user to specify a
    /// > value, but I haven't yet figured out how those work because it is
    /// > wrapped up in the simba SIMD crate in complicated macros.
    /// >
    /// > Open to suggestions about how this can be better handled, or the
    /// > try_normalize function can just be removed for now.
    fn try_normalize(self, minimum_magnitude: Self::NumericType) -> Option<Self>;
}

impl<T> Vector2DOps for Coord<T>
where
    T: CoordFloat + Send + Sync,
{
    type NumericType = T;

    fn wedge_product(self, right: Coord<T>) -> Self::NumericType {
        self.x * right.y - self.y * right.x
    }

    fn dot_product(self, other: Self) -> Self::NumericType {
        self.x * other.x + self.y * other.y
    }

    fn magnitude(self) -> Self::NumericType {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn magnitude_squared(self) -> Self::NumericType {
        self.x * self.x + self.y * self.y
    }

    fn left(self) -> Self {
        Self {
            x: -self.y,
            y: self.x,
        }
    }

    fn right(self) -> Self {
        Self {
            x: self.y,
            y: -self.x,
        }
    }

    fn try_normalize(self, minimum_magnitude: Self::NumericType) -> Option<Self> {
        let magnitude = self.magnitude();
        if magnitude.is_finite() && magnitude.abs() > minimum_magnitude {
            Some(self / magnitude)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    // crate dependencies
    use crate::coord;

    // private imports
    use super::Vector2DOps;

    #[test]
    fn test_cross_product() {
        // perpendicular unit length
        let a = coord! { x: 1f64, y: 0f64 };
        let b = coord! { x: 0f64, y: 1f64 };

        // expect the area of parallelogram
        assert_eq!(a.wedge_product(b), 1f64);
        // expect swapping will result in negative
        assert_eq!(b.wedge_product(a), -1f64);

        // Add skew; expect results should be the same
        let a = coord! { x: 1f64, y: 0f64 };
        let b = coord! { x: 1f64, y: 1f64 };

        // expect the area of parallelogram
        assert_eq!(a.wedge_product(b), 1f64);
        // expect swapping will result in negative
        assert_eq!(b.wedge_product(a), -1f64);

        // Make Colinear; expect zero
        let a = coord! { x: 2f64, y: 2f64 };
        let b = coord! { x: 1f64, y: 1f64 };
        assert_eq!(a.wedge_product(b), 0f64);
    }

    #[test]
    fn test_dot_product() {
        // perpendicular unit length
        let a = coord! { x: 1f64, y: 0f64 };
        let b = coord! { x: 0f64, y: 1f64 };
        // expect zero for perpendicular
        assert_eq!(a.dot_product(b), 0f64);

        // Parallel, same direction
        let a = coord! { x: 1f64, y: 0f64 };
        let b = coord! { x: 2f64, y: 0f64 };
        // expect +ive product of magnitudes
        assert_eq!(a.dot_product(b), 2f64);
        // expect swapping will have same result
        assert_eq!(b.dot_product(a), 2f64);

        // Parallel, opposite direction
        let a = coord! { x: 3f64, y: 4f64 };
        let b = coord! { x: -3f64, y: -4f64 };
        // expect -ive product of magnitudes
        assert_eq!(a.dot_product(b), -25f64);
        // expect swapping will have same result
        assert_eq!(b.dot_product(a), -25f64);
    }

    #[test]
    fn test_magnitude() {
        let a = coord! { x: 1f64, y: 0f64 };
        assert_eq!(a.magnitude(), 1f64);

        let a = coord! { x: 0f64, y: 0f64 };
        assert_eq!(a.magnitude(), 0f64);

        let a = coord! { x: -3f64, y: 4f64 };
        assert_eq!(a.magnitude(), 5f64);
    }

    #[test]
    fn test_magnitude_squared() {
        let a = coord! { x: 1f64, y: 0f64 };
        assert_eq!(a.magnitude_squared(), 1f64);

        let a = coord! { x: 0f64, y: 0f64 };
        assert_eq!(a.magnitude_squared(), 0f64);

        let a = coord! { x: -3f64, y: 4f64 };
        assert_eq!(a.magnitude_squared(), 25f64);
    }

    #[test]
    fn test_left_right() {
        let a = coord! { x: 1f64, y: 0f64 };
        let a_left = coord! { x: 0f64, y: 1f64 };
        let a_right = coord! { x: 0f64, y: -1f64 };

        assert_eq!(a.left(), a_left);
        assert_eq!(a.right(), a_right);
        assert_eq!(a.left(), -a.right());
    }

    #[test]
    fn test_left_right_match_rotate() {
        use crate::algorithm::rotate::Rotate;
        use crate::Point;
        // The aim of this test is to confirm that wording in documentation is
        // consistent.

        // when the user is in a coordinate system where the y axis is flipped
        // (eg screen coordinates in a HTML canvas), then rotation directions
        // will be different to those described in the documentation.

        // The documentation for the Rotate trait says: 'Positive angles are
        // counter-clockwise, and negative angles are clockwise rotations'

        let counter_clockwise_rotation_degrees = 90.0;
        let clockwise_rotation_degrees = -counter_clockwise_rotation_degrees;

        let a: Point = coord! { x: 1.0, y: 0.0 }.into();
        let origin: Point = coord! { x: 0.0, y: 0.0 }.into();

        // left is anti-clockwise
        assert_relative_eq!(
            Point::from(a.0.left()),
            a.rotate_around_point(counter_clockwise_rotation_degrees, origin),
        );
        // right is clockwise
        assert_relative_eq!(
            Point::from(a.0.right()),
            a.rotate_around_point(clockwise_rotation_degrees, origin),
        );
    }

    #[test]
    fn test_normalize() {
        let a = coord! { x: 1.0, y: 0.0 };
        assert_relative_eq!(a.try_normalize(2e-301f64).unwrap(), a);

        let a = coord! { x: 1.0 / f64::sqrt(2.0), y: -1.0 / f64::sqrt(2.0) };
        assert_relative_eq!(a.try_normalize(2e-301f64).unwrap(), a);

        let a = coord! { x: -10.0, y: 8.0 };
        assert_relative_eq!(
            a.try_normalize(2e-301f64).unwrap(),
            a / f64::sqrt(10.0 * 10.0 + 8.0 * 8.0)
        );

        let a = coord! { x: 0.0, y: 1e-301f64 };
        assert_eq!(a.try_normalize(2e-301f64), None);
    }
}
