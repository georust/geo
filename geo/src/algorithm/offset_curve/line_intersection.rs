/// Definitions used in documentation for this module;
/// 
/// - **line**:
///   - The straight path on a plane that
///   - extends infinitely in both directions.
///   - defined by two distinct points `a` and `b` and
///   - has the direction of the vector from `a` to `b`
/// 
/// - **segment**:
///   - A finite portion of a `line` which
///   - lies between the points `a` and `b`
///   - has the direction of the vector from `a` to `b`
/// 
/// - **ray**:
///   - A segment which extends infinitely in the forward direction
/// 
/// - `Line`: the type [crate::Line] which is actually a **segment**.

use super::vector_extensions::VectorExtensions;
use crate::{
    Coord,
    CoordFloat,
    CoordNum,
    // algorithm::kernels::Kernel,
    // algorithm::kernels::RobustKernel,
    // Orientation
};

/// Used to encode the relationship between a **segment** and an intersection
/// point. See documentation for [LineIntersectionResultWithRelationships]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub(super) enum LineSegmentIntersectionType {
    /// The intersection point lies between the start and end of the **segment**
    ///
    /// Abbreviated to `TIP` in original paper
    TrueIntersectionPoint,
    /// The intersection point is 'false' or 'virtual': it lies on the same
    /// **line** as the **segment**, but not between the start and end points of
    /// the **segment**.
    ///
    /// Abbreviated to `FIP` in original paper
    
    // Note: Rust does not permit nested enum declaration, so
    // FalseIntersectionPointType has to be declared below. 
    FalseIntersectionPoint(FalseIntersectionPointType),
}

/// These are the variants of [LineSegmentIntersectionType::FalseIntersectionPoint]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub(super) enum FalseIntersectionPointType {
    /// The intersection point is 'false' or 'virtual': it lies on the same
    /// **line** as the **segment**, and before the start of the **segment**.
    ///
    /// Abbreviated to `NFIP` in original paper (Negative)
    /// (also referred to as `FFIP` in Figure 6, but i think this is an 
    ///  error?)
    BeforeStart,
    /// The intersection point is 'false' or 'virtual': it lies on the same
    /// **line** as the **segment**,  and after the end of the **segment**.
    ///
    /// Abbreviated to `PFIP` in original paper (Positive)
    AfterEnd,
}


/// Struct to contain the result for [line_segment_intersection_with_relationships]
#[derive(Clone, Debug)]
pub(super) struct LineIntersectionResultWithRelationships<T>
where
    T: CoordNum,
{
    pub ab: LineSegmentIntersectionType,
    pub cd: LineSegmentIntersectionType,
    pub intersection: Coord<T>,
}

/// Computes the intersection between two line segments;
/// a to b (`ab`), and c to d (`cd`)
///
/// > note: looks like there is already `cartesian_intersect` as a private
/// > method in simplifyvw.rs. It uses the orient2d method of [Kernel],
/// > however it only gives a true/false answer and does not return the
/// > intersection point or parameters needed.
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

fn line_segment_intersection_with_parameters<T>(
    a: Coord<T>,
    b: Coord<T>,
    c: Coord<T>,
    d: Coord<T>,
) -> Option<(T, T, Coord<T>)>
where
    T: CoordFloat,
{
    let ab = b - a;
    let cd = d - c;
    let ac = c - a;

    let ab_cross_cd = ab.cross_product_2d(cd);
    if T::is_zero(&ab_cross_cd) {
        // Segments are exactly parallel or colinear
        None
    } else {
        // Division by zero is prevented, but testing is needed to see what
        // happens for near-parallel sections of line.
        let t_ab =  ac.cross_product_2d(cd) / ab_cross_cd;
        let t_cd = -ab.cross_product_2d(ac) / ab_cross_cd;
        let intersection = a + ab * t_ab;

        Some((t_ab, t_cd, intersection))
    }

    // TODO:
    // The above could be replaced with the following, but at the cost of
    // repeating some computation.

    // match RobustKernel::orient2d(*a, *b, *d) {
    //     Orientation::Collinear => None,
    //     _ => {
    //         let t_ab = cross_product_2d(ac, cd) / ab_cross_cd;
    //         let t_cd = -cross_product_2d(ab, ac) / ab_cross_cd;
    //         let intersection = *a + ab * t_ab;
    //         Some((t_ab, t_cd, intersection))
    //     }
    // }
}

/// This is a simple wrapper for [line_segment_intersection_with_parameters];
/// Returns the intersection point as well as the relationship between the point
/// and each of the input line segments. See [LineSegmentIntersectionType]
pub(super) fn line_segment_intersection_with_relationships<T>(
    a: Coord<T>,
    b: Coord<T>,
    c: Coord<T>,
    d: Coord<T>,
) -> Option<LineIntersectionResultWithRelationships<T>>
where
    T: CoordFloat,
{
    line_segment_intersection_with_parameters(a, b, c, d).map(|(t_ab, t_cd, intersection)| {
        use FalseIntersectionPointType::{AfterEnd, BeforeStart};
        use LineSegmentIntersectionType::{FalseIntersectionPoint, TrueIntersectionPoint};
        LineIntersectionResultWithRelationships {
            ab: if T::zero() <= t_ab && t_ab <= T::one() {
                TrueIntersectionPoint
            } else if t_ab < T::zero() {
                FalseIntersectionPoint(BeforeStart)
            } else {
                FalseIntersectionPoint(AfterEnd)
            },
            cd: if T::zero() <= t_cd && t_cd <= T::one() {
                TrueIntersectionPoint
            } else if t_cd < T::zero() {
                FalseIntersectionPoint(BeforeStart)
            } else {
                FalseIntersectionPoint(AfterEnd)
            },
            intersection,
        }
    })
}

#[cfg(test)]
mod test {
    use super::{
        line_segment_intersection_with_parameters, line_segment_intersection_with_relationships,
        FalseIntersectionPointType, LineIntersectionResultWithRelationships,
        LineSegmentIntersectionType,
    };
    use crate::{Coord};
    use FalseIntersectionPointType::{AfterEnd, BeforeStart};
    use LineSegmentIntersectionType::{FalseIntersectionPoint, TrueIntersectionPoint};

    #[test]
    fn test_line_segment_intersection_with_parameters() {
        let a = Coord { x: 0f64, y: 0f64 };
        let b = Coord { x: 2f64, y: 2f64 };
        let c = Coord { x: 0f64, y: 1f64 };
        let d = Coord { x: 1f64, y: 0f64 };
        if let Some((t_ab, t_cd, intersection)) =
            line_segment_intersection_with_parameters(a, b, c, d)
        {
            assert_eq!(t_ab, 0.25f64);
            assert_eq!(t_cd, 0.50f64);
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
    fn test_line_segment_intersection_with_parameters_parallel() {
        let a = Coord { x: 3f64, y: 4f64 };
        let b = Coord { x: 6f64, y: 8f64 };
        let c = Coord { x: 9f64, y: 9f64 };
        let d = Coord { x: 12f64, y: 13f64 };
        assert_eq!(
            line_segment_intersection_with_parameters(a, b, c, d),
            None
        )
    }
    #[test]
    fn test_line_segment_intersection_with_parameters_colinear() {
        let a = Coord { x: 1f64, y: 2f64 };
        let b = Coord { x: 2f64, y: 4f64 };
        let c = Coord { x: 3f64, y: 6f64 };
        let d = Coord { x: 5f64, y: 10f64 };
        assert_eq!(
            line_segment_intersection_with_parameters(a, b, c, d),
            None
        )
    }

    #[test]
    fn test_line_segment_intersection_with_relationships() {
        let a = Coord { x: 1f64, y: 2f64 };
        let b = Coord { x: 2f64, y: 3f64 };
        let c = Coord { x: 0f64, y: 2f64 };
        let d = Coord { x: -2f64, y: 6f64 };

        let expected_intersection_point = Coord { x: 1f64 / 3f64, y: 4f64 / 3f64 };

        if let Some(LineIntersectionResultWithRelationships {
            ab,
            cd,
            intersection,
        }) = line_segment_intersection_with_relationships(a, b, c, d)
        {
            assert_eq!(ab, FalseIntersectionPoint(BeforeStart));
            assert_eq!(cd, FalseIntersectionPoint(BeforeStart));
            assert_relative_eq!(intersection, expected_intersection_point);
        } else {
            assert!(false);
        }

        if let Some(LineIntersectionResultWithRelationships {
            ab,
            cd,
            intersection,
        }) = line_segment_intersection_with_relationships(b, a, c, d)
        {
            assert_eq!(ab, FalseIntersectionPoint(AfterEnd));
            assert_eq!(cd, FalseIntersectionPoint(BeforeStart));
            assert_relative_eq!(intersection, expected_intersection_point);
        } else {
            assert!(false);
        }

        if let Some(LineIntersectionResultWithRelationships {
            ab,
            cd,
            intersection,
        }) = line_segment_intersection_with_relationships(a, b, d, c)
        {
            assert_eq!(ab, FalseIntersectionPoint(BeforeStart));
            assert_eq!(cd, FalseIntersectionPoint(AfterEnd));
            assert_relative_eq!(intersection, expected_intersection_point);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_line_segment_intersection_with_relationships_true() {
        let a = Coord { x: 0f64, y: 1f64 };
        let b = Coord { x: 2f64, y: 3f64 };
        let c = Coord { x: 0f64, y: 2f64 };
        let d = Coord { x: -2f64, y: 6f64 };

        let expected_intersection_point = Coord { x: 1f64 / 3f64, y: 4f64 / 3f64 };

        if let Some(LineIntersectionResultWithRelationships {
            ab,
            cd,
            intersection,
        }) = line_segment_intersection_with_relationships(a, b, c, d)
        {
            assert_eq!(ab, TrueIntersectionPoint);
            assert_eq!(cd, FalseIntersectionPoint(BeforeStart));
            assert_relative_eq!(intersection, expected_intersection_point);
        } else {
            assert!(false);
        }
    }
}
