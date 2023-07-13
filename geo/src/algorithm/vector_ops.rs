//! This module defines the [Vector2DOps] trait and implements it for the
//! [Coord] struct.

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
    type Scalar: CoordNum + Send + Sync;

    /// The euclidean distance between this coordinate and the origin
    ///
    /// `sqrt(x² + y²)`
    ///
    fn magnitude(self) -> Self::Scalar;

    /// The squared distance between this coordinate and the origin.
    /// (Avoids the square root calculation when it is not needed)
    ///
    /// `x² + y²`
    ///
    fn magnitude_squared(self) -> Self::Scalar;

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
    fn dot_product(self, other: Rhs) -> Self::Scalar;

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
    fn wedge_product(self, other: Rhs) -> Self::Scalar;

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
    fn try_normalize(self, minimum_magnitude: Self::Scalar) -> Option<Self>;
}

impl<T> Vector2DOps for Coord<T>
where
    T: CoordFloat + Send + Sync,
{
    type Scalar = T;

    fn wedge_product(self, right: Coord<T>) -> Self::Scalar {
        self.x * right.y - self.y * right.x
    }

    fn dot_product(self, other: Self) -> Self::Scalar {
        self.x * other.x + self.y * other.y
    }

    fn magnitude(self) -> Self::Scalar {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn magnitude_squared(self) -> Self::Scalar {
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

    fn try_normalize(self, minimum_magnitude: Self::Scalar) -> Option<Self> {
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
        let f64_minimum_threshold = 2e-301f64;
        // Already Normalized
        let a = coord! {
            x: 1.0,
            y: 0.0
        };
        assert_relative_eq!(a.try_normalize(f64_minimum_threshold).unwrap(), a);

        // Already Normalized
        let a = coord! {
            x: 1.0 / f64::sqrt(2.0),
            y: -1.0 / f64::sqrt(2.0)
        };
        assert_relative_eq!(a.try_normalize(f64_minimum_threshold).unwrap(), a);

        // Non trivial example
        let a = coord! { x: -10.0, y: 8.0 };
        assert_relative_eq!(
            a.try_normalize(f64_minimum_threshold).unwrap(),
            coord! { x: -10.0, y: 8.0 } / f64::sqrt(10.0 * 10.0 + 8.0 * 8.0)
        );

        // Very Small Input - Fails because below threshold
        let a = coord! {
            x: 0.0,
            y: f64_minimum_threshold / 2.0
        };
        assert_eq!(a.try_normalize(f64_minimum_threshold), None);

        // Zero vector - Fails because below threshold
        let a = coord! { x: 0.0, y: 0.0 };
        assert_eq!(a.try_normalize(f64_minimum_threshold), None);

        // Large vector Pass;
        // Note: this fails because the magnitude calculation overflows to infinity
        //       this could be avoided my modifying the implementation to use scaling be avoided using scaling
        let a = coord! {
            x: f64::sqrt(f64::MAX/2.0),
            y: f64::sqrt(f64::MAX/2.0)
        };
        assert!(a.try_normalize(f64_minimum_threshold).is_some());

        // Large Vector Fail;
        // Note: This test uses the unstable float_next_up_down feature
        // let a = coord! { x: f64::sqrt(f64::MAX/2.0), y: f64::next_up(f64::sqrt(f64::MAX/2.0)) };
        // assert!(a.try_normalize(1e-10f64).is_none());

        // NaN
        let a = coord! { x: f64::NAN, y: 0.0 };
        assert_eq!(a.try_normalize(f64_minimum_threshold), None);

        // Infinite
        let a = coord! { x: f64::INFINITY, y: 0.0 };
        assert_eq!(a.try_normalize(f64_minimum_threshold), None);
    }
}
