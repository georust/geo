use num_traits::Float;
use types::LineString;

pub fn twice_area<T>(linestring: &LineString<T>) -> T where T: Float {
    if linestring.0.is_empty() || linestring.0.len() == 1 {
        return T::zero();
    }
    let mut tmp = T::zero();
    for ps in linestring.0.windows(2) {
        tmp = tmp + (ps[0].x() * ps[1].y() - ps[1].x() * ps[0].y());
    }

    tmp
}
