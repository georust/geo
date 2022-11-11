/// # Offset - Signed Cartesian
/// 
///  ## Utility Functions
/// 
/// This module starts by defining a heap of private utility functions 
/// [dot_product], [cross_product], [magnitude], [normalize], [rescale]
///
/// It looks like some of these are already implemented on  stuff implemented 
/// on the Point struct; but that feels misplaced to me?
/// 
/// Looks like they might eventually belong in the Kernel trait??
/// 
/// For my first pull request I'll just implement them functional style and keep
/// the damage to one module ;)

use geo_types::Coord;
use crate::{kernels::RobustKernel, CoordFloat, CoordNum, Kernel, Line, LineString, Orientation};


/// 2D Dot Product
fn dot_product<T>(left: Coord<T>, right: Coord<T>) -> T
where
    T: CoordNum,
{
    left.x * right.x + left.y * right.y
}

/// 2D "Cross Product"
/// 
/// If we pretend the `z` ordinate is zero we can still use the 3D cross product
/// on 2D vectors and various useful properties still hold (e.g. it is still the
/// area of the parallelogram formed by the two input vectors)
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
/// 
/// Note: `cross_prod` is already defined on Point... but that it seems to be
/// some other operation on 3 points
fn cross_product<T>(left: Coord<T>, right: Coord<T>) -> T
where
    T: CoordNum,
{
    left.x * right.y - left.y * right.x
}

/// Compute the magnitude of a Coord<T> as if it was a vector
fn magnitude<T>(a: Coord<T>) -> T
where
    T: CoordFloat,
{
    (a.x * a.x + a.y * a.y).sqrt()
}

/// Return a new coord in the same direction as the old coord but
/// with a magnitude of 1 unit
/// no protection from divide by zero
fn normalize<T>(a: Coord<T>) -> Coord<T>
where
    T: CoordFloat,
{
    a.clone() / magnitude(a)
}

/// Return a new coord in the same direction as the old coord but with the
/// specified magnitude
/// No protection from divide by zero
fn rescale<T>(a: Coord<T>, new_magnitude: T) -> Coord<T>
where
    T: CoordFloat,
{
    normalize(a) * new_magnitude
}

/// Struct to contain the result for [line_intersection_with_parameter]
struct LineIntersectionWithParameterResult<T>
where
    T: CoordNum,
{
    t_ab: T,
    t_cd: T,
    intersection: Coord<T>,
}

/// Computes the intersection between two line segments;
/// a to b (`ab`), and c to d (`cd`)
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
/// The determinant of the matrix `M` is the reciprocal of the cross product
/// of `ab` and `cd`.
///
/// ```text
/// 1/(ab×cd)
/// ```
///
/// Therefore if `ab×cd = 0` the determinant is undefined and the matrix cannot
/// be inverted this means the lines are either
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
/// [ t_ab] = [ ac×cd / ab×cd ]
/// [-t_cd] = [ ab×ac / ab×cd ]
/// ```

fn line_intersection_with_parameter<T>(
    a:Coord<T>,
    b:Coord<T>,
    c:Coord<T>,
    d:Coord<T>,
) -> LineIntersectionWithParameterResult<T>
where
    T: CoordFloat,
{

    let ab = b - a;
    let cd = d - c;
    let ac = c - a;

    let ab_cross_cd = cross_product(ab, cd);

    if ab_cross_cd == num_traits::zero() {
        // TODO: We can't tolerate this situation as it will cause a divide by
        //       zero in the next step. Even values close to zero are a problem,
        //       but I don't know how to deal with that problem jut yet
        todo!("")
    }

    let t_ab = cross_product(ac, cd) / ab_cross_cd;
    let t_cd = cross_product(ac, cd) / ab_cross_cd;
    let intersection = a + rescale(ab, t_ab);
    LineIntersectionWithParameterResult { t_ab, t_cd, intersection }
}

/// Signed offset of Geometry assuming cartesian coordinate system.
/// 
/// This is a cheap offset algorithm that is suitable for flat coordinate systems
/// (or if your lat/lon data is near the equator)
///
/// My Priority for implementing the trait is as follows:
/// - Line<impl CoordFloat>
/// - LineString<impl CoordFloat>
/// - MultiLineString<impl CoordFloat>
/// - ... maybe some closed shapes like triangle, polygon?
///
/// The following are a list of known limitations,
/// some may be removed during development,
/// others are very hard to fix.
///
/// - No checking for zero length input.
///   Invalid results may be caused by division by zero.
/// - No check is implemented to prevent execution if the specified offset
///   distance is zero.
/// - Only local cropping where the output is self-intersecting.
///   Non-adjacent line segments in the output may be self-intersecting.
/// - There is no mitre-limit; A LineString which
///   doubles back on itself will produce an elbow at infinity
pub trait OffsetSignedCartesian<T>
where
    T: CoordNum,
{
    fn offset(&self, distance: T) -> Self;
}

impl<T> OffsetSignedCartesian<T> for Line<T>
where
    T: CoordFloat,
{
    fn offset(&self, distance: T) -> Self {
        let delta = self.delta();
        let len = (delta.x * delta.x + delta.y * delta.y).sqrt();
        let delta = Coord {
            x: delta.y / len,
            y: -delta.x / len,
        };
        Line::new(self.start + delta * distance, self.end + delta * distance)
    }
}

impl<T> OffsetSignedCartesian<T> for LineString<T>
where
    T: CoordFloat,
{
    fn offset(&self, distance: T) -> Self {
        if self.0.len() < 2 {
            // TODO: Fail on invalid input
            return self.clone();
        }
        let offset_segments: Vec<Line<T>> =
            self.lines().map(|item| item.offset(distance)).collect();
        if offset_segments.len() == 1 {
            return offset_segments[0].into();
        }
        let x = offset_segments[0];
        // Guess that the output has the same number of vertices as the input.
        // It is a safe bet for inputs with oblique bends and long segments;
        let mut raw_offset_ls: Vec<Coord<T>> = Vec::with_capacity(self.0.len());
        raw_offset_ls.push(offset_segments[0].start.clone());
        // safe, non-copy `pairwise` iterator is not a thing in rust because
        // the borrow checker is triggered by too much fun.
        // The itertools crate has a `tuple_windows` function
        // (it does a clone of every element which is probably fine?)
        // Extra dependencies are naff so we sacrifice the readability of
        // iterators and go for an old style for loop;
        for i in 0..offset_segments.len() - 1usize {
            let line_ab = offset_segments[i];
            let line_cd = offset_segments[i + 1usize];

            let a = line_ab.start;
            let b = line_ab.end;
            let c = line_cd.start;
            let d = line_cd.end;

            let ab = b-a;
            let cd = d-c;

            // check for colinear case
            // This is a flakey check with potentially unsafe type cast :/
            if <f64 as num_traits::NumCast>::from(cross_product(ab, cd)).unwrap() < 0.0000001f64 { 
                raw_offset_ls.push(b);
                continue;
            }
            // TODO: Do we need the full overhead of RobustKernel for this?
            //       The simple kernel impl seems to be blank?
            // if RobustKernel::orient2d(ab.start, ab.end, cd.end) == Orientation::Collinear {
            //     raw_offset_ls.push(ab.end);
            //     continue;
            // }

            let LineIntersectionWithParameterResult{t_ab, t_cd, intersection} = line_intersection_with_parameter(a, b, c, d);

            let TIP_ab  = num_traits::zero::<T>() <= t_ab && t_ab <= num_traits::one();
			let FIP_ab  = ! TIP_ab;
			let PFIP_ab = FIP_ab && t_ab > num_traits::zero();
			
			
			let TIP_cd = num_traits::zero::<T>() <= t_cd && t_cd <= num_traits::one();
			let FIP_cd = ! TIP_cd;
			
			
			if TIP_ab && TIP_cd {
				// Case 2a
				// TODO: test for mitre limit
				raw_offset_ls.push(intersection);
            } else if FIP_ab && FIP_cd{
				// Case 2b.
				if PFIP_ab {
					// TODO: test for mitre limit
					raw_offset_ls.push(intersection);
                } else {
					raw_offset_ls.push(b);
					raw_offset_ls.push(c);
                }
            } else {
				// Case 2c. (either ab or cd
                raw_offset_ls.push(b);
                raw_offset_ls.push(c);
            }
	        raw_offset_ls.push(d)  
        }
        todo!("Not done yet.")
    }
}

#[cfg(test)]
mod test {
    // crate dependencies
    use crate::{line_string, Coord, Line};

    // private imports
    use super::{
        cross_product,
        OffsetSignedCartesian
    };

    #[test]
    fn test_cross_product(){
        let a = Coord{x:0f64, y:0f64};
        let b = Coord{x:0f64, y:1f64};
        let c = Coord{x:1f64, y:0f64};

        let ab = b - a;
        let ac = c - a;

        
        // expect the area of the parallelogram
        assert_eq!(
            cross_product(ac, ab),
            1f64
        );
        // expect swapping will result in negative
        assert_eq!(
            cross_product(ab, ac),
            -1f64
        );

        // Add skew
        let a = Coord{x:0f64, y:0f64};
        let b = Coord{x:0f64, y:1f64};
        let c = Coord{x:1f64, y:1f64};

        let ab = b - a;
        let ac = c - a;

        // expect the area of the parallelogram
        assert_eq!(
            cross_product(ac, ab),
            1f64
        );
        // expect swapping will result in negative
        assert_eq!(
            cross_product(ab, ac),
            -1f64
        );
    }

    #[test]
    fn offset_line_test() {
        let line = Line::new(Coord { x: 1f64, y: 1f64 }, Coord { x: 1f64, y: 2f64 });
        let actual_result = line.offset(1.0);
        assert_eq!(
            actual_result,
            Line::new(Coord { x: 2f64, y: 1f64 }, Coord { x: 2f64, y: 2f64 },)
        );
    }
    #[test]
    fn offset_line_test_negative() {
        let line = Line::new(Coord { x: 1f64, y: 1f64 }, Coord { x: 1f64, y: 2f64 });
        let actual_result = line.offset(-1.0);
        assert_eq!(
            actual_result,
            Line::new(Coord { x: 0f64, y: 1f64 }, Coord { x: 0f64, y: 2f64 },)
        );
    }
}
