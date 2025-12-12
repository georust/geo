use crate::CoordNum;
use geo_types::Coord;

/// for debug assert
pub(crate) fn is_monotone<T: CoordNum>(coords: &[Coord<T>]) -> bool {
    if coords.len() <= 2 {
        return true;
    }

    let increasing_x = coords.windows(2).all(|w| w[0].x >= w[1].x);
    let decreasing_x = coords.windows(2).all(|w| w[0].x <= w[1].x);
    let monotonic_x = increasing_x || decreasing_x;

    let increasing_y = coords.windows(2).all(|w| w[0].y >= w[1].y);
    let decreasing_y = coords.windows(2).all(|w| w[0].y <= w[1].y);
    let monotonic_y = increasing_y || decreasing_y;

    monotonic_x && monotonic_y
}
