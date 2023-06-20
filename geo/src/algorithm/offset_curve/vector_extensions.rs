use crate::{Coord, CoordFloat};

/// Extends the `Coord` struct with more vector operations;
///
/// - [VectorExtensions::cross_product_2d],
/// - [VectorExtensions::magnitude],
/// - [VectorExtensions::magnitude_squared],
/// - [VectorExtensions::dot_product],
/// - [VectorExtensions::left],
/// - [VectorExtensions::right],
///
/// > Note: I implemented these functions here because I am trying to localize
/// > my changes to the [crate::algorithm::offset] module for the time being.
///
/// > I think I remember seeing some open issues and pull requests that will
/// > hopefully make this trait unnecessary.
///
/// TODO: make a list
///
/// > Also there is the [crate::algorithm::Kernel] trait which has some default
/// > implementations very similar to this trait. This is definitely a
/// > re-invented wheel,
///
/// > Probably better to try to contribute to the existing structure of the code
/// > though rather than suggest disruptive changes.... buuuuut I'm still
/// > feeling this way of implementing [VectorExtensions] on the [Coord] struct
/// > is not entirely indefensible...? Perhaps there could be a
/// > `VectorExtensionsRobust`?
/// > Just thinking aloud.

pub(super) trait VectorExtensions<Rhs = Self>
where
    Self: Copy,
{
    type NumericType;
    /// The signed magnitude of the 3D "Cross Product" assuming z ordinates are
    /// zero
    ///
    /// > Note: [geo_types::Point::cross_prod()] is already defined on
    /// >       [geo_types::Point]... but that it seems to be some other
    /// >       operation on 3 points??
    ///
    /// > Note: Elsewhere in this project the cross product seems to be done
    /// >       inline and is referred to as 'determinant' since it is the same
    /// >       as the determinant of a 2x2 matrix.
    ///
    /// > Note: The [geo_types::Line] struct also has a
    /// > [geo_types::Line::determinant()] function which is the same as
    /// > `cross_product_2d(line.start, line.end)`
    ///
    ///
    /// If we pretend the `z` ordinate is zero we can still use the 3D cross
    /// product on 2D vectors and various useful properties still hold:
    ///
    /// - the magnitude is the signed area of the parallelogram formed by the
    ///   two input vectors;
    /// - the sign depends on the order of the operands and their clockwise /
    ///   anti-clockwise orientation with respect to the origin (is b to the
    ///   left or right of the line between the origin and a)
    /// - if the two input points are colinear with the origin, the magnitude is
    ///   zero
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
    /// We can define the 2D cross product as the magnitude of the 3D cross product
    /// as follows
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

    /// Rotate this coordinate around the origin in the xy plane 90 degrees
    /// anti-clockwise (Consistent with [crate::algorithm::rotate::Rotate]).
    fn left(self) -> Self;

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
        // expect + product of magnitudes
        assert_eq!(a.dot_product(b), 2f64);
        // expect swapping will have same result
        assert_eq!(b.dot_product(a), 2f64);

        // Parallel, opposite direction
        let a = Coord { x: 3f64, y: 4f64 };
        let b = Coord { x: -3f64, y: -4f64 };
        // expect - product of magnitudes
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
    }

    #[test]
    fn test_left_right_match_rotate() {
        use crate::algorithm::rotate::Rotate;
        use crate::Point;
        // the documentation for the Rotate trait says: 'Positive angles are
        // counter-clockwise, and negative angles are clockwise rotations'
        // left is anti-clockwise and right is clockwise: check that the results
        // match:

        let a: Point = Coord { x: 1f64, y: 0f64 }.into();

        assert_relative_eq!(
            a.0.right(),
            a.rotate_around_point(-90.0, Coord { x: 0.0, y: 0.0 }.into())
                .0
        );
        assert_relative_eq!(
            a.0.left(),
            a.rotate_around_point(90.0, Coord { x: 0.0, y: 0.0 }.into())
                .0
        );
    }
}
