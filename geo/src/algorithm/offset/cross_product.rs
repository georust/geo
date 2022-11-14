use crate::CoordFloat;
use geo_types::Coord;

/// 2D "Cross Product"
///
/// > Note: `cross_prod` is already defined on `Point`... but that it seems to be
/// >       some other operation on 3 points
///
/// > Note: Elsewhere in this project the cross product seems to be done inline
/// >       and is referred to as 'determinant' since it is the same as the
/// >       determinant of a 2x2 matrix.
///
/// If we pretend the `z` ordinate is zero we can still use the 3D cross product
/// on 2D vectors and various useful properties still hold (e.g. it is still the
/// signed area of the parallelogram formed by the two input vectors, with the
/// sign being dependant on the order and properties of the inputs)
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
pub(super) fn cross_product<T>(left: Coord<T>, right: Coord<T>) -> T
where
    T: CoordFloat,
{
    left.x * right.y - left.y * right.x
}

#[cfg(test)]
mod test {
    // crate dependencies
    use crate::Coord;

    // private imports
    use super::cross_product;

    #[test]
    fn test_cross_product() {
        let a = Coord { x: 0f64, y: 0f64 };
        let b = Coord { x: 0f64, y: 1f64 };
        let c = Coord { x: 1f64, y: 0f64 };

        let ab = b - a;
        let ac = c - a;

        // expect the area of the parallelogram
        assert_eq!(cross_product(ac, ab), 1f64);
        // expect swapping will result in negative
        assert_eq!(cross_product(ab, ac), -1f64);

        // Add skew
        let a = Coord { x: 0f64, y: 0f64 };
        let b = Coord { x: 0f64, y: 1f64 };
        let c = Coord { x: 1f64, y: 1f64 };

        let ab = b - a;
        let ac = c - a;

        // expect the area of the parallelogram
        assert_eq!(cross_product(ac, ab), 1f64);
        // expect swapping will result in negative
        assert_eq!(cross_product(ab, ac), -1f64);
    }
}
