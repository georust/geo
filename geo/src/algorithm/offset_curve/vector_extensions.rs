use crate::{Coord, CoordFloat};

/// Extends the `Coord` struct with some common vector operations;
///
/// - [VectorExtensions::cross_product_2d],
/// - [VectorExtensions::magnitude],
/// - [VectorExtensions::magnitude_squared],
/// - [VectorExtensions::dot_product],
/// - [VectorExtensions::left],
/// - [VectorExtensions::right],
///
/// TODO: I implemented these functions here because I am trying to localize
/// my changes to the [crate::algorithm::offset_curve] module at the moment.
///
/// The [crate::algorithm::Kernel] trait has some functions which overlap with
/// this trait. I have realized I am re-inventing the wheel here.

pub(super) trait VectorExtensions<Rhs = Self>
where
    Self: Copy,
{
    type NumericType;
    /// The 2D cross product is the signed magnitude of the 3D "Cross Product"
    /// assuming z ordinates are zero.
    /// 
    /// ## Other names for this function:
    /// 
    /// - In exterior algebra, it is called the wedge product.
    /// - If the inputs are packed into a 2x2 matrix, this is the determinant.
    ///
    /// ## Other appearances in this library
    /// 
    /// 1. [geo_types::Point::cross_prod()] is already defined on
    ///    [geo_types::Point]... but that it seems to be some other
    ///    operation on 3 points??
    ///
    /// 2. Note: The [geo_types::Line] struct also has a
    ///    [geo_types::Line::determinant()] function which is the same as
    ///    `cross_product_2d(line.start, line.end)`
    /// 
    /// 3. The [crate::algorithm::Kernel::orient2d()] trait default
    ///    implementation uses cross product to compute orientation. It returns
    ///    an enum, not the numeric value which is needed for line segment
    ///    intersection.
    ///
    ///
    /// ## Properties
    ///
    /// - The absolute value of the cross product is the area of the
    ///   parallelogram formed by the operands
    /// - The sign of the output is reversed if the operands are reversed
    /// - The sign can be used to check if the operands are clockwise /
    ///   anti-clockwise orientation with respect to the origin;
    ///   or phrased differently:
    ///   "is b to the left of the line between the origin and a"?
    /// - If the operands are colinear with the origin, the magnitude is zero
    ///
    /// 
    /// ## Derivation
    /// 
    /// From basis vectors `i`,`j`,`k` and the axioms on wikipedia
    /// [Cross product](https://en.wikipedia.org/wiki/Cross_product#Computing);
    ///
    /// ```text
    /// i×j = k
    /// j×k = i
    /// k×i = j
    ///
    /// j×i = -k
    /// k×j = -i
    /// i×k = -j
    ///
    /// i×i = j×j = k×k = 0
    /// ```
    ///
    /// We can define the 2D cross product as the magnitude of the 3D cross
    /// product as follows
    ///
    /// ```text
    /// |a × b| = |(a_x·i + a_y·j + 0·k) × (b_x·i + b_y·j + 0·k)|
    ///         = |a_x·b_x·(i×i) + a_x·b_y·(i×j) + a_y·b_x·(j×i) + a_y·b_y·(j×j)|
    ///         = |a_x·b_x·( 0 ) + a_x·b_y·( k ) + a_y·b_x·(-k ) + a_y·b_y·( 0 )|
    ///         = |               (a_x·b_y       - a_y·b_x)·k |
    ///         =                  a_x·b_y       - a_y·b_x
    /// ```
    fn cross_product_2d(self, other: Rhs) -> Self::NumericType;

    /// The inner product of the coordinate components
    ///
    /// ```ignore
    /// self.x*other.x + self.y*other.y
    /// ```
    fn dot_product(self, other: Rhs) -> Self::NumericType;

    /// The euclidean distance between this coordinate and the origin
    ///
    /// ```
    /// (self.x*self.x + self.y*self.y).sqrt()
    /// ```
    fn magnitude(self) -> Self::NumericType;

    /// The squared distance between this coordinate and the origin.
    /// (Avoids the square root calculation when it is not needed)
    ///
    /// ```
    /// self.x*self.x + self.y*self.y
    /// ```
    fn magnitude_squared(self) -> Self::NumericType;

    /// In a coordinate system where positive is up and to the right;
    /// Rotate this coordinate around the origin in the xy plane 90 degrees
    /// anti-clockwise (Consistent with [crate::algorithm::rotate::Rotate]).
    fn left(self) -> Self;

    /// In a coordinate system where positive is up and to the right;
    /// Rotate this coordinate around the origin in the xy plane 90 degrees
    /// clockwise (Consistent with [crate::algorithm::rotate::Rotate]).
    fn right(self) -> Self;
}

impl<T> VectorExtensions for Coord<T>
where
    T: CoordFloat,
{
    type NumericType = T;

    fn cross_product_2d(self, right: Coord<T>) -> Self::NumericType {
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
}

#[cfg(test)]
mod test {
    // crate dependencies
    use crate::Coord;

    // private imports
    use super::VectorExtensions;

    #[test]
    fn test_cross_product() {
        // perpendicular unit length
        let a = Coord { x: 1f64, y: 0f64 };
        let b = Coord { x: 0f64, y: 1f64 };

        // expect the area of parallelogram
        assert_eq!(a.cross_product_2d(b), 1f64);
        // expect swapping will result in negative
        assert_eq!(b.cross_product_2d(a), -1f64);

        // Add skew; expect results should be the same
        let a = Coord { x: 1f64, y: 0f64 };
        let b = Coord { x: 1f64, y: 1f64 };

        // expect the area of parallelogram
        assert_eq!(a.cross_product_2d(b), 1f64);
        // expect swapping will result in negative
        assert_eq!(b.cross_product_2d(a), -1f64);

        // Make Colinear; expect zero
        let a = Coord { x: 2f64, y: 2f64 };
        let b = Coord { x: 1f64, y: 1f64 };
        assert_eq!(a.cross_product_2d(b), 0f64);
    }

    #[test]
    fn test_dot_product() {
        // perpendicular unit length
        let a = Coord { x: 1f64, y: 0f64 };
        let b = Coord { x: 0f64, y: 1f64 };
        // expect zero for perpendicular
        assert_eq!(a.dot_product(b), 0f64);

        // Parallel, same direction
        let a = Coord { x: 1f64, y: 0f64 };
        let b = Coord { x: 2f64, y: 0f64 };
        // expect +ive product of magnitudes
        assert_eq!(a.dot_product(b), 2f64);
        // expect swapping will have same result
        assert_eq!(b.dot_product(a), 2f64);

        // Parallel, opposite direction
        let a = Coord { x: 3f64, y: 4f64 };
        let b = Coord { x: -3f64, y: -4f64 };
        // expect -ive product of magnitudes
        assert_eq!(a.dot_product(b), -25f64);
        // expect swapping will have same result
        assert_eq!(b.dot_product(a), -25f64);
    }

    #[test]
    fn test_magnitude() {
        let a = Coord { x: 1f64, y: 0f64 };
        assert_eq!(a.magnitude(), 1f64);

        let a = Coord { x: 0f64, y: 0f64 };
        assert_eq!(a.magnitude(), 0f64);

        let a = Coord { x: -3f64, y: 4f64 };
        assert_eq!(a.magnitude(), 5f64);
    }

    #[test]
    fn test_magnitude_squared() {
        let a = Coord { x: 1f64, y: 0f64 };
        assert_eq!(a.magnitude_squared(), 1f64);

        let a = Coord { x: 0f64, y: 0f64 };
        assert_eq!(a.magnitude_squared(), 0f64);

        let a = Coord { x: -3f64, y: 4f64 };
        assert_eq!(a.magnitude_squared(), 25f64);
    }

    #[test]
    fn test_left_right() {
        let a = Coord { x: 1f64, y: 0f64 };
        let a_left = Coord { x: 0f64, y: 1f64 };
        let a_right = Coord { x: 0f64, y: -1f64 };

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

        let a: Point = Coord { x: 1.0, y: 0.0 }.into();
        let origin:Point = Coord::<f64>::zero().into();

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
}
