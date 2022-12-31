use crate::{haversine_distance::HaversineDistance, Bearing};
use crate::{HaversineDestination, Point, MEAN_EARTH_RADIUS, CoordsIter};
use geo_types::{CoordFloat, Line, LineString};
use num_traits::FromPrimitive;

/// Closest point between two geometries using great circles.
///
/// https://edwilliams.org/avform147.htm#XTE
///
/// For a great circle segment that crosses the pole
pub trait HaversineClosestPoint<T, Pt = Self>
where
    T: CoordFloat + FromPrimitive,
{
    fn haversine_closest_point(&self, from: &Pt) -> Option<Point<T>>;
}

impl<T> HaversineClosestPoint<T, Point<T>> for Point<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn haversine_closest_point(&self, pt: &Point<T>) -> Option<Point<T>> {
        return Some(*pt);
    }
}

impl<T> HaversineClosestPoint<T, Point<T>> for Line<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn haversine_closest_point(&self, from: &Point<T>) -> Option<Point<T>> {
        let p1 = self.start_point();
        let p2 = self.end_point();

        // This can probably be done cheaper
        let d3 = p2.haversine_distance(&p1);
        if d3 <= T::epsilon() {
            // "Degenerate" segment, return either p1 or p2
            return Some(p1);
        }

        let pi = T::from(std::f64::consts::PI).unwrap();
        let crs_ad = p1.bearing(*from).to_radians();
        let crs_ab = p1.bearing(p2).to_radians();
        let crs_ba = if crs_ab > T::zero() {
            crs_ab - pi
        } else {
            crs_ab + pi
        };
        let crs_bd = p2.bearing(*from).to_radians();
        let d_crs1 = crs_ad - crs_ab;
        let d_crs2 = crs_bd - crs_ba;

        let d1 = p1.haversine_distance(from);

        // d1, d2, d3 are in principle not needed, only the sign matters
        let projection1 = d_crs1.cos();
        let projection2 = d_crs2.cos();

        if projection1.is_sign_positive() && projection2.is_sign_positive() {
            let earth_radius = T::from(MEAN_EARTH_RADIUS).unwrap();
            let xtd = (((d1 / earth_radius).sin() * d_crs1.sin()).asin()).abs();
            let atd = earth_radius * (((d1 / earth_radius).cos() / xtd.cos()).acos()).abs();
            return Some(p1.haversine_destination(crs_ab.to_degrees(), atd));
        }

        // Projected falls outside the GC Arc
        // Return shortest distance pt, project either on point sp1 or sp2
        let d2 = p2.haversine_distance(from);
        if d1 < d2 {
            return Some(p1);
        }
        return Some(p2);
    }
}

impl<T> HaversineClosestPoint<T, Point<T>> for LineString<T>
where
    T: CoordFloat + FromPrimitive,
{
    // This is a naive implementation
    fn haversine_closest_point(&self, from: &Point<T>) -> Option<Point<T>> {
        if self.coords_count() == 0 {
            return None
        }

        let mut min_distance = T::max_value();
        let mut rv: Option<Point<T>> = None;
        for line in self.lines() {
            if let Some(pt) = line.haversine_closest_point(from) {
                let dist = pt.haversine_distance(from);

                if dist < min_distance {
                    min_distance = dist;
                    rv = Some(pt);
                }
            }
        }

        rv
    }
}

#[cfg(test)]
mod test {
    trait ApproxEq<T> {
        fn approx_eq(&self, other: &Self) -> bool;
    }

    impl<T: CoordFloat + FromPrimitive + approx::RelativeEq> ApproxEq<T> for Point<T> {
        fn approx_eq(&self, other: &Self) -> bool {
            relative_eq!(self.x(), other.x()) && relative_eq!(self.y(), other.y())
        }
    }

    use wkt::TryFromWkt;

    use super::*;

    #[test]
    fn point_to_point() {
        let p_1 = Point::new(-84.74905, 32.61454);
        let p_2 = Point::new(-85.93942, 32.11055);
        assert_eq!(p_1.haversine_closest_point(&p_2), Some(p_2));
        assert_eq!(p_2.haversine_closest_point(&p_1), Some(p_1));
    }

    #[test]
    fn point_to_line_1() {
        let p_1 = Point::new(-84.74905, 32.61454);
        let p_2 = Point::new(-85.93942, 32.11055);
        let line = Line::new(p_2, p_1);

        let p_from = Point::new(-84.75625, 31.81056);

        assert_eq!(
            line.haversine_closest_point(&p_from),
            Some(Point::new(-85.13337428852164, 32.45365659858937))
        );

        let p_from = Point::new(-85.67211, 32.39774);
        if let Some(pt) = line.haversine_closest_point(&p_from) {
            assert!(pt.approx_eq(&Point::new(-85.58999680564376, 32.26023534389268)));
        } else {
            assert!(false);
        }
    }

    // Across the pole
    #[test]
    fn point_to_line_across_equator() {
        let p_1 = Point::new(-38.42479487179491571, 75.13738846153847817);
        let p_2 = Point::new(-28.60871282051286357, -85.27805769230766941);
        let line = Line::new(p_2, p_1);
        let p_from = Point::new(-25.86062, -87.32053);

        if let Some(pt) = line.haversine_closest_point(&p_from) {
            assert!(pt.approx_eq(&Point::new(-28.60871282051286357, -85.27805769230766941)));
        } else {
            assert!(false)
        }
    }

    #[test]
    fn point_to_line_across_close_to_north_pole() {
        let p_1 = Point::new(-37.24492187499998863, 79.50861250000002656);
        let p_2 = Point::new(50.59687500000001137, 81.05462812500002201);
        let line = Line::new(p_2, p_1);
        let p_from = Point::new(8.15172, 77.40041);

        match line.haversine_closest_point(&p_from) {
            Some(pt) => assert!(pt.approx_eq(&Point::new(5.48109492316554, 82.99828098761533))),
            None => assert!(false),
        }
    }

    #[test]
    fn point_to_linestring() {
        let wkt = "LineString (3.86503906250000284 11.71231367187503736, 9.48691406250000568 17.3341886718750402, \
            13.28167968750000227 15.50707929687503395, 15.95207031249999829 9.18246992187503963, \
            7.73007812500000568 8.33918867187503565, 16.0926171875000108 2.8578605468750311, \
            23.26050781250000909 6.3715324218750311, 24.66597656250000625 14.24215742187503508, \
            20.23875000000001023 13.6799699218750419, 19.11437500000000966 10.72848554687503508, \
            18.20082031249999943 13.60969648437503565, 16.79535156250000227 17.54500898437503054, \
            20.09820312500001194 17.26391523437503395, 22.27667968750000682 15.64762617187503224, \
            24.24433593750001137 18.24774335937503622, 18.97382812500001137 18.38829023437503452)";

        let linestring = LineString::try_from_wkt_str(wkt).unwrap();

        let p_from = Point::new(17.02374, 10.57037);

        match linestring.haversine_closest_point(&p_from) {
            Some(pt) => assert!(pt.approx_eq(&Point::new(15.611386947136054, 10.006831648991811))),
            None => assert!(false),
        }
    }

    #[test]
    fn point_to_empty_linestring() {
        let linestring = LineString::new(vec![]);

        let p_from = Point::new(17.02374, 10.57037);

        match linestring.haversine_closest_point(&p_from) {
            Some(_) => assert!(false),
            None => {},
        }
    }
}
