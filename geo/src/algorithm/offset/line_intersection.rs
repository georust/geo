use super::cross_product::cross_product_2d;
use crate::{CoordFloat, CoordNum};
use geo_types::Coord;

/// Struct to contain the result for [line_intersection_with_parameter]
pub(super) struct LineIntersectionWithParameterResult<T>
where
    T: CoordNum,
{
    pub t_ab: T,
    pub t_cd: T,
    pub intersection: Coord<T>,
}

/// Computes the intersection between two line segments;
/// a to b (`ab`), and c to d (`cd`)
///
/// > note: looks like there is already `cartesian_intersect` as a private
/// > method in simplifyvw.rs. It is nice because it uses the orient2d method
/// > of the Kernel, however it only gives a true/false answer and does not
/// > return the intersection point or parameters needed.
///
/// We already have LineIntersection trait BUT we need a function that also
/// returns the parameters for both lines described below. The LineIntersection
/// trait uses some fancy unrolled code it seems unlikely it could be adapted
/// for this purpose.
///
/// Returns the intersection point **and** parameters `t_ab` and `t_cd`
/// described below
///
/// The intersection of segments can be expressed as a parametric equation
/// where `t_ab` and `t_cd` are unknown scalars :
///
/// ```text
/// a + ab · t_ab = c + cd · t_cd
/// ```
///
/// > note: a real intersection can only happen when `0 <= t_ab <= 1` and
/// >       `0 <= t_cd <= 1` but this function will find intersections anyway
/// >       which may lay outside of the line segments
///
/// This can be rearranged as follows:
///
/// ```text
/// ab · t_ab - cd · t_cd = c - a
/// ```
///
/// Collecting the scalars `t_ab` and `-t_cd` into the column vector `T`,
/// and by collecting the vectors `ab` and `cd` into matrix `M`:
/// we get the matrix form:
///
/// ```text
/// [ab_x  cd_x][ t_ab] = [ac_x]
/// [ab_y  cd_y][-t_cd]   [ac_y]
/// ```
///
/// or
///
/// ```text
/// M·T=ac
/// ```
///
/// Inverting the matrix `M` involves taking the reciprocal of the determinant
/// (the determinant is same as the of the [cross_product()] of `ab` and `cd`)
///
/// ```text
/// 1/(ab×cd)
/// ```
///
/// Therefore if `ab×cd = 0` the determinant is undefined and the matrix cannot
/// be inverted. The lines are either
///   a) parallel or
///   b) collinear
///
/// Pre-multiplying both sides by the inverted 2x2 matrix we get:
///
/// ```text
/// [ t_ab] = 1/(ab×cd) · [ cd_y  -cd_x][ac_x]
/// [-t_cd]               [-ab_y   ab_x][ac_y]
/// ```
///
/// or
///
/// ```text
/// T = M⁻¹·ac
/// ```
///
/// Expands to:
///
/// ```text
/// [ t_ab] = 1/(ab_x·cd_y - ab_y·cd_x)·[ cd_y·ac_x - cd_x·ac_y]
/// [-t_cd]                             [-ab_y·ac_x + ab_x·ac_y]
/// ```
///
/// Since it is tidier to write cross products, observe that the above is
/// equivalent to:
///
/// ```text
/// [t_ab] = [   ac×cd / ab×cd ]
/// [t_cd] = [ - ab×ac / ab×cd ]
/// ```

pub(super) fn line_intersection_with_parameter<T>(
    a: &Coord<T>,
    b: &Coord<T>,
    c: &Coord<T>,
    d: &Coord<T>,
) -> Option<LineIntersectionWithParameterResult<T>>
where
    T: CoordFloat,
{
    let ab = *b - *a;
    let cd = *d - *c;
    let ac = *c - *a;

    // TODO: I'm still confused about how to use Kernel / RobustKernel;
    //       the following did not work. I need to read more code
    //       from the rest of this repo to understand.
    // if Kernel::orient2d(*a, *b, *d) == Orientation::Collinear {
    //       note that it is sufficient to check that only one of
    //       c or d are colinear with ab because of how they are
    //       related by the original line string.
    // TODO: The following line
    //       - Does not use the Kernel
    //       - uses an arbitrary threshold value which needs more thought
    let ab_cross_cd = cross_product_2d(ab, cd);
    if <f64 as num_traits::NumCast>::from(ab_cross_cd)
        .unwrap()
        .abs()
        < num_traits::cast(0.0000001f64).unwrap()
    {
        // Segments are parallel or colinear
        None
    } else {
        let t_ab = cross_product_2d(ac, cd) / ab_cross_cd;
        let t_cd = -cross_product_2d(ab, ac) / ab_cross_cd;
        let intersection = *a + ab * t_ab;
        Some(LineIntersectionWithParameterResult {
            t_ab,
            t_cd,
            intersection,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Coord;
    #[test]
    fn test_intersection() {
        let a = Coord { x: 0f64, y: 0f64 };
        let b = Coord { x: 2f64, y: 2f64 };
        let c = Coord { x: 0f64, y: 1f64 };
        let d = Coord { x: 1f64, y: 0f64 };
        if let Some(LineIntersectionWithParameterResult {
            t_ab,
            t_cd,
            intersection,
        }) = line_intersection_with_parameter(&a, &b, &c, &d)
        {
            assert_eq!(t_ab, 0.25f64);
            assert_eq!(t_cd, 0.5f64);
            assert_eq!(
                intersection,
                Coord {
                    x: 0.5f64,
                    y: 0.5f64
                }
            );
        } else {
            assert!(false)
        }
    }

    #[test]
    fn test_intersection_colinear() {
        let a = Coord { x: 3f64, y: 4f64 };
        let b = Coord { x: 6f64, y: 8f64 };
        let c = Coord { x: 7f64, y: 7f64 };
        let d = Coord { x: 10f64, y: 9f64 };
        if let Some(LineIntersectionWithParameterResult {
            t_ab,
            t_cd,
            intersection,
        }) = line_intersection_with_parameter(&a, &b, &c, &d)
        {
            assert_eq!(t_ab, 0.25f64);
            assert_eq!(t_cd, 0.5f64);
            assert_eq!(
                intersection,
                Coord {
                    x: 0.5f64,
                    y: 0.5f64
                }
            );
        } else {
            assert!(false)
        }
    }
}
