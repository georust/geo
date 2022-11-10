use geo_types::Coord;

use crate::{
    CoordFloat, CoordNum, Geometry, GeometryCollection, Line, LineString, MultiLineString,
    MultiPoint, MultiPolygon, Point, Polygon, Rect, Triangle, kernels::Kernel,
};

/// Signed offset of Geometry assuming cartesian coordinate system.
/// This is a cheap offset algorithim that is suitable for flat coordinate systems (or if your lat/lon data is near the equator)
///
/// My Priority for implementing the trait is as follows:
/// - Line<impl CoordFloat>
/// - LineString<impl CoordFloat>
/// - MultiLineString<impl CoordFloat>
/// - ... maybe some other easy ones like triangle, polygon
/// 
/// The following are a list of known limitations,
/// some may be removed during development,
/// others are very hard to fix.
///
/// - No checking for zero length input.
///   Invalid results may be caused by division by zero.
/// - Only local cropping where the output is self-intersecting.
///   Non-adjacent line segments in the output may be self-intersecting.
/// - There is no mitre-limit; A LineString which doubles back on itself will produce an elbow at infinity
pub trait OffsetSignedCartesian<T>
where
    T: CoordNum,
{
    fn offset(&self, distance:T) -> Self;
}

impl<T> OffsetSignedCartesian<T> for Line<T> where T: CoordFloat {
    fn offset(&self, distance:T) -> Self{
        let delta = self.delta();
        let len = (delta.x * delta.x + delta.y * delta.y).sqrt();
        let delta  = Coord{
            x:delta.y/len,
            y:-delta.x/len
        };
        Line::new(
            self.start+delta*distance,
            self.end+delta*distance
        )
        
    }
}

// impl<T> Offset<T> for LineString<T> where T: CoordFloat {
//     fn offset(&self, distance:T) -> Self{
//         let offset_segments = self.lines().map(|item|item.offset(distance));
//         let raw_offset_ls:Vec<Coordinate<T>>;
//         todo!("Working form here")
//     }
// }


#[cfg(test)]
mod test {
    use crate::algorithm::offset_signed_cartesian::OffsetSignedCartesian;
    use crate::{line_string, Line, Coord};
    #[test]
    fn offset_line_test(){
        let line = Line::new(
            Coord{x:1f64, y:1f64},
              Coord{x:1f64, y:2f64},
        );
        let actual_result = line.offset(1.0);
        assert_eq!(
            actual_result,
            Line::new(
                Coord{x:2f64, y:1f64},
                Coord{x:2f64, y:2f64},
            )
        );
    }
    #[test]
    fn offset_line_test_negative(){
        let line = Line::new(
            Coord{x:1f64, y:1f64},
            Coord{x:1f64, y:2f64}
        );
        let actual_result = line.offset(-1.0);
        assert_eq!(
            actual_result,
            Line::new(
                Coord{x:0f64, y:1f64},
                Coord{x:0f64, y:2f64},
            )
        );
    }
}