use num_traits::FromPrimitive;

use geo_types::{CoordFloat, Line, Point};

use crate::{CrossTrackDistance, HaversineBearing, HaversineDistance};

pub trait HaversineDistanceToLine<T, Rhs> {
    fn haversine_distance_to_line(&self, line: &Rhs) -> T;
}

impl<T: CoordFloat + FromPrimitive> HaversineDistanceToLine<T, Line<T>> for Point<T> {
    fn haversine_distance_to_line(&self, line: &Line<T>) -> T {
        let lines_start = Point::from(line.start);
        let lines_end = Point::from(line.end);

        if is_inner_angle_between_points_obtuse(self, &lines_start, &lines_end) {
            self.haversine_distance(&lines_start)
        } else if is_inner_angle_between_points_obtuse(self, &lines_end, &lines_start) {
            self.haversine_distance(&lines_end)
        } else {
            self.cross_track_distance(&lines_start, &lines_end)
        }
    }
}

fn is_inner_angle_between_points_obtuse<T: CoordFloat + FromPrimitive>(
    a: &Point<T>,
    b: &Point<T>,
    c: &Point<T>,
) -> bool {
    is_angle_obtuse(inner_angle_between_three_points(a, b, c))
}

fn is_angle_obtuse<T: CoordFloat + FromPrimitive>(angle: T) -> bool {
    angle > T::from(90.).unwrap()
}

fn inner_angle_between_three_points<T: CoordFloat + FromPrimitive>(
    a: &Point<T>,
    b: &Point<T>,
    c: &Point<T>,
) -> T {
    let angle_180 = T::from(180.).unwrap();
    let angle_360 = T::from(360.).unwrap();
    let bearing_bc = b.haversine_bearing(*c);
    let bearing_ba = b.haversine_bearing(*a);

    let angle = (bearing_bc - bearing_ba).abs();

    if angle <= angle_180 {
        angle
    } else {
        angle_360 - angle
    }
}

#[cfg(test)]
mod test {
    use geo_types::Line;

    use crate::HaversineDistance;
    use crate::HaversineDistanceToLine;
    use crate::Point;

    #[test]
    fn haversine_distance_to_line_pointing_through_point() {
        let p = Point::new(0., 0.);
        let line_point_a = Point::new(1., 0.);
        let line_point_b = Point::new(2., 0.);

        assert_relative_eq!(
            p.haversine_distance_to_line(&Line::new(line_point_a, line_point_b)),
            p.haversine_distance(&line_point_a),
            epsilon = 1.0e-6
        );
    }

    #[test]
    fn haversine_distance_to_line_orthogonal_to_point() {
        let p = Point::new(0., 0.);
        let line_point_a = Point::new(1., -1.);
        let line_point_b = Point::new(1., 1.);

        assert_relative_eq!(
            p.haversine_distance_to_line(&Line::new(line_point_a, line_point_b)),
            p.haversine_distance(&Point::new(1., 0.)),
            epsilon = 1.0e-6
        );

        assert_relative_eq!(
            p.haversine_distance_to_line(&Line::new(line_point_b, line_point_a)),
            p.haversine_distance(&Point::new(1., 0.)),
            epsilon = 1.0e-6
        );
    }

    #[test]
    fn haversine_distance_to_line_where_start_is_nearest_to_point() {
        let p = Point::new(0., 0.);
        let line_point_a = Point::new(1., 1.);
        let line_point_b = Point::new(2., 1.);

        assert_relative_eq!(
            p.haversine_distance_to_line(&Line::new(line_point_a, line_point_b)),
            p.haversine_distance(&line_point_a),
            epsilon = 1.0e-6
        );
    }

    #[test]
    fn haversine_distance_to_line_where_end_is_nearest_to_point() {
        let p = Point::new(0., 0.);
        let line_point_a = Point::new(-2., 1.);
        let line_point_b = Point::new(-1., 1.);

        assert_relative_eq!(
            p.haversine_distance_to_line(&Line::new(line_point_a, line_point_b)),
            p.haversine_distance(&line_point_b),
            epsilon = 1.0e-6
        );
    }
}
