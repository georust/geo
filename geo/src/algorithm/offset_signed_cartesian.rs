

use geo_types::Coord;

use crate::{
    Orientation, kernels::RobustKernel, CoordFloat, CoordNum, Geometry, GeometryCollection, Kernel, Line,
    LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon, Rect, Triangle
};

/// 2D Dot Product
/// TODO: Not sure where to put this
/// it looks like there is some stuff implemented on Point; but that feels misplaced to me?
/// I would have implemented it on Coord<T>
/// but for my first pull request I trying to keep the damage to one file

fn dot<T>(a:Coord<T>, b: Coord<T>) -> T where T: CoordNum {
    a.x * b.x + a.y * b.y
}

/// 2D "Cross Product"
/// 
/// If we pretend the `z` ordinate is zero we can still use the 3D cross product on 2D vectors and various useful properties still hold.
/// It is useful to simplify line segment intersection
/// Note: `cross_prod` is already defined on Point... but that it seems to be some other operation on 3 points
/// 
/// TODO: make some tests to confirm the negative is in the right place
fn cross<T>(a:Coord<T>, b:Coord<T>) -> T where T:CoordNum {
    a.x*b.y - a.y*b.x
}

/// Compute the magnitude of a Coord<T> as if it was a vector
fn magnitude<T>(a:Coord<T>) -> T where T:CoordFloat{
    (a.x*a.x+a.y*a.y).sqrt()
}

/// Return a new coord in the same direction as the old coord but with a magnitude of 1 unit
/// no protection from divide by zero
fn normalize<T>(a:Coord<T>) -> Coord<T> where T:CoordFloat{
    a.clone()/magnitude(a)
}

/// Return a new coord in the same direction as the old coord but with the specified magnitude
/// No protection from divide by zero
fn rescale<T>(a:Coord<T>, new_magnitude: T) -> Coord<T> where T:CoordFloat{
    normalize(a) * new_magnitude
}

/// computes the intersection between two line segments; a to b, and c to d
/// 
/// The intersection of segments can be expressed as a parametric equation
/// where t1 and t2 are unknown scalars 
/// 
/// ```text
/// a + ab·t1 = c + cd·t2
/// ```
/// 
/// > note: a real intersection can only happen when `0<=t1<=1 and 0<=t2<=1`
///
/// This can be rearranged as follows:
/// 
/// ```text
/// ab·t1 - cd·t2 = c - a
/// ```
///
/// by collecting the scalars t1 and -t2 into the column vector T,
/// and by collecting the vectors ab and cd into matrix M:
/// we get the matrix form:
///
/// ```text
/// [ab_x  cd_x][ t1] = [ac_x]
/// [ab_y  cd_y][-t2]   [ac_y]
/// ```
/// 
/// or
/// 
/// ```text
/// M·T=ac
/// ```
/// 
/// the determinant of the matrix M is the inverse of the cross product of ab and cd.
/// 
/// ```text
/// 1/(ab×cd)
/// ```
/// 
/// Therefore if ab×cd=0 the determinant is undefined and the matrix cannot be inverted
/// This means the lines are
///   a) parallel and
///   b) possibly collinear
///
/// pre-multiplying both sides by the inverted 2x2 matrix we get:
/// 
/// ```text
/// [ t1] = 1/(ab×cd)·[ cd_y  -cd_x][ac_x]
/// [-t2]             [-ab_y   ab_x][ac_y]
/// ```
/// 
/// or
/// 
/// ```text
/// T = M⁻¹·ac
/// ```
/// 
/// multiplied out
/// 
/// ```text
/// [ t1] = 1/(ab_x·cd_y - ab_y·cd_x)·[ cd_y·ac_x - cd_x·ac_y]
/// [-t2]                             [-ab_y·ac_x + ab_x·ac_y]
/// ```
/// 
/// since it is neater to write cross products, observe that the above is equivalent to:
/// 
/// ```text
/// [ t1] = [ ac×cd / ab×cd ]
/// [-t2] = [ ab×ac / ab×cd ]
/// ```
struct LineItersectionWithParameterResult<T> where T : CoordNum{
    t_ab:T,
    t_cd:T,
    point:Coord<T>,
}

fn line_intersection_with_parameter<T>(line_ab:Line<T>, line_cd:Line<T>) -> LineItersectionWithParameterResult<T> where T: CoordFloat{
    // TODO: we already have line intersection trait
    //       but i require the parameters for both lines
    //       which can be obtained at the same time
    //       This is a much dirtier algorithm;
    let a = line_ab.start;
    let b = line_ab.end;
    let c = line_cd.start;
    let d = line_cd.end;

    let ab = b - a;
    let cd = d - c;
    let ac = c - a;

    let ab_cross_cd = cross(ab, cd);

    if ab_cross_cd == num_traits::zero() {
        // TODO: We can't tolerate this situation as it will cause a divide by zero in the next step
        //       Even values close to zero are a problem, but I don't know how to deal with that problem jut yet
        todo!("")
    }


    let t_ab = cross(ac,cd) / ab_cross_cd;
    let t_cd = cross(ac,cd) / ab_cross_cd;
    let point  = a +  rescale(ab, t_ab);
    LineItersectionWithParameterResult{
        t_ab,
        t_cd,
        point
    }
    
}

/// Signed offset of Geometry assuming cartesian coordinate system.
/// This is a cheap offset algorithim that is suitable for flat coordinate systems (or if your lat/lon data is near the equator)
///
/// My Priority for implementing the trait is as follows:
/// - Line<impl CoordFloat>
/// - LineString<impl CoordFloat>
/// - MultiLineString<impl CoordFloat>
/// - ... maybe some closed shapes easy ones like triangle, polygon
///
/// The following are a list of known limitations,
/// some may be removed during development,
/// others are very hard to fix.
///
/// - No checking for zero length input.
///   Invalid results may be caused by division by zero.
/// - No check is implemented to prevent execution if the specified offset distance is zero
/// - Only local cropping where the output is self-intersecting.
///   Non-adjacent line segments in the output may be self-intersecting.
/// - There is no mitre-limit; A LineString which doubles back on itself will produce an elbow at infinity
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
        // if self.0.len() < 2 {
        //     // TODO: Fail on invalid input
        //     return self.clone();
        // }
        // let offset_segments: Vec<Line<T>> =
        //     self.lines().map(|item| item.offset(distance)).collect();
        // if offset_segments.len() == 1 {
        //     return offset_segments[0].into();
        // }
        // let x = offset_segments[0];
        // // TODO: we make a bet that the output has the same number of verticies as the input plus 1
        // // It is a safe bet for inputs with oblique bends and long segments; we could try `self.0.len() + 5` or something and
        // // do some benchmarks on real examples to see if it is worth guessing a bit larger.
        // let raw_offset_ls: Vec<Coord<T>> = Vec::with_capacity(self.0.len());
        // raw_offset_ls.push(offset_segments[0].start);
        // // the `pairwise` iterator is not a thing in rust because it makes the borrow checker sad :(,
        // // the itertools crate has a similar tuple_windows function,
        // // it does a clone of every element which is probably fine
        // // for now we dont want to add that dependancy so we sacrifice
        // // the readability of iterators and go for an old style for loop
        // for i in 0..offset_segments.len() - 1usize {
        //     let ab = offset_segments[i];
        //     let cd = offset_segments[i + 1usize];
        //     // TODO: Do I need to use the RobustKernel for this? The simple kernel has no impl
        //     //       (it has a comment which says SimpleKernel says it is for integer types?
        //     //        I could add an impl to SimpleKernel for float types that does a faster check?)
        //     //       We really want to prevent "very close to parallel" segments as this will cause
        //     //       nonsense for our line segment intersection algorithim
        //     if RobustKernel::orient2d(ab.start, ab.end, cd.d) == Orientation::Collinear {
        //         raw_offset_ls.push(ab.end);
        //         continue;
        //     }


        // }
        todo!("Not done yet. I need to make a commit here to keep my progress")
    }
}

#[cfg(test)]
mod test {
    use crate::algorithm::offset_signed_cartesian::OffsetSignedCartesian;
    use crate::{line_string, Coord, Line};
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
