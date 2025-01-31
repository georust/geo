use crate::line_measures::{Bearing, Destination, Distance, Haversine};
use crate::{Closest, Contains};
use crate::{CoordsIter, GeoFloat, Point, MEAN_EARTH_RADIUS};
use geo_types::{
    Coord, Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Polygon, Rect, Triangle,
};

use num_traits::FromPrimitive;

/// Calculates the closest `Point` on a geometry from a given `Point` in sperical coordinates.
///
/// Similar to [`ClosestPoint`](crate::ClosestPoint) but for spherical coordinates:
/// * Longitude (x) in the [-180; 180] degrees range.
/// * Latitude (y) in the [-90; 90] degrees range.
///
/// The implementation is based on <https://edwilliams.org/avform147.htm#XTE>.
///
/// See [`Closest<F>`] for a description of the return states.
///
/// Note: This may return `Closest::Intersection` even for non-intersecting geometies if they are
/// very close to the input.
///
/// Example:
/// ```
/// # use geo::HaversineClosestPoint;
/// # use geo::{Point, Line, Closest};
/// use approx::assert_relative_eq;
/// let line = Line::new(Point::new(-85.93942, 32.11055), Point::new(-84.74905, 32.61454));
/// let p_from = Point::new(-84.75625, 31.81056);
/// if let Closest::SinglePoint(pt) = line.haversine_closest_point(&p_from) {
///     assert_relative_eq!(pt, Point::new(-85.13337428852164, 32.45365659858937), epsilon = 1e-6);
/// } else {
///     panic!("Closest::SinglePoint expected");
/// }
/// ```
pub trait HaversineClosestPoint<T>
where
    T: GeoFloat + FromPrimitive,
{
    fn haversine_closest_point(&self, from: &Point<T>) -> Closest<T>;
}

// Implement for references as well as types
impl<T, G> HaversineClosestPoint<T> for &'_ G
where
    G: HaversineClosestPoint<T>,
    T: GeoFloat + FromPrimitive,
{
    fn haversine_closest_point(&self, from: &Point<T>) -> Closest<T> {
        (*self).haversine_closest_point(from)
    }
}

impl<T> HaversineClosestPoint<T> for Point<T>
where
    T: GeoFloat + FromPrimitive,
{
    fn haversine_closest_point(&self, pt: &Point<T>) -> Closest<T> {
        if self == pt {
            Closest::Intersection(*self)
        } else {
            Closest::SinglePoint(*self)
        }
    }
}

impl<T> HaversineClosestPoint<T> for Coord<T>
where
    T: GeoFloat + FromPrimitive,
{
    fn haversine_closest_point(&self, pt: &Point<T>) -> Closest<T> {
        Point::from(*self).haversine_closest_point(pt)
    }
}

impl<T> HaversineClosestPoint<T> for Line<T>
where
    T: GeoFloat + FromPrimitive,
{
    fn haversine_closest_point(&self, from: &Point<T>) -> Closest<T> {
        let p1 = self.start_point();
        let p2 = self.end_point();

        // Optimization if the point s exactly one of the ends of the arc.
        if p1 == *from {
            return Closest::Intersection(p1);
        }

        if p2 == *from {
            return Closest::Intersection(p2);
        }

        // This can probably be done cheaper
        let d3 = Haversine.distance(p2, p1);
        if d3 <= T::epsilon() {
            // I think here it should be return Closest::SinglePoint(p1)
            // If the line segment is degenerated to a point, that point is still the closest
            // (instead of indeterminate as in the Cartesian case).

            return Closest::SinglePoint(p1);
        }

        let pi = T::from(std::f64::consts::PI).unwrap();
        let crs_ad = Haversine.bearing(p1, *from).to_radians();
        let crs_ab = Haversine.bearing(p1, p2).to_radians();
        let crs_ba = if crs_ab > T::zero() {
            crs_ab - pi
        } else {
            crs_ab + pi
        };
        let crs_bd = Haversine.bearing(p2, *from).to_radians();
        let d_crs1 = crs_ad - crs_ab;
        let d_crs2 = crs_bd - crs_ba;

        let d1 = Haversine.distance(p1, *from);

        // d1, d2, d3 are in principle not needed, only the sign matters
        let projection1 = d_crs1.cos();
        let projection2 = d_crs2.cos();

        if projection1.is_sign_positive() && projection2.is_sign_positive() {
            let earth_radius = T::from(MEAN_EARTH_RADIUS).unwrap();
            let xtd = (((d1 / earth_radius).sin() * d_crs1.sin()).asin()).abs();
            let atd = earth_radius * (((d1 / earth_radius).cos() / xtd.cos()).acos()).abs();

            if xtd < T::epsilon() {
                return Closest::Intersection(*from);
            } else {
                return Closest::SinglePoint(Haversine.destination(p1, crs_ab.to_degrees(), atd));
            }
        }

        // Projected falls outside the GC Arc
        // Return shortest distance pt, project either on point sp1 or sp2
        let d2 = Haversine.distance(p2, *from);
        if d1 < d2 {
            return Closest::SinglePoint(p1);
        }
        Closest::SinglePoint(p2)
    }
}

impl<T> HaversineClosestPoint<T> for LineString<T>
where
    T: GeoFloat + FromPrimitive,
{
    // This is a naive implementation
    fn haversine_closest_point(&self, from: &Point<T>) -> Closest<T> {
        if self.coords_count() == 0 {
            return Closest::Indeterminate; // Empty LineString
        }

        let mut min_distance = num_traits::Float::max_value();
        let mut rv = Closest::Indeterminate;

        for line in self.lines() {
            match line.haversine_closest_point(from) {
                intersect @ Closest::Intersection(_) => {
                    // let's investigate the situation here:
                    // - we have discovered that the point actually intersects the linestring.
                    // Clearly no other non-intersecting point can be closer now.
                    // Even if the linestring is degenerate and is intersecting itself at this specific point
                    // by definition it must be that exact point.
                    // So instead of returning an indeterminate of exactly the same point, we just return here.
                    return intersect;
                }
                Closest::SinglePoint(pt) => {
                    let dist = Haversine.distance(pt, *from);
                    if dist < min_distance {
                        min_distance = dist;
                        rv = Closest::SinglePoint(pt);
                    }
                }
                // If there is a case where we cannot figure out the closest,
                // Then it needs to be intereminate, instead of skipping
                Closest::Indeterminate => return Closest::Indeterminate,
            }
        }

        rv
    }
}

fn closest_closed_simple_poly<T, I>(lines: I, from: &Point<T>) -> (Closest<T>, T)
where
    T: GeoFloat + FromPrimitive,
    I: IntoIterator<Item = Line<T>>,
{
    let mut min_distance = num_traits::Float::max_value();
    let mut rv = Closest::Indeterminate;
    for line in lines {
        match line.haversine_closest_point(from) {
            intersect @ Closest::Intersection(_) => {
                // same as for the linestring, even if we detected multiple intersections,
                // they would by definition be the same point, so we can just return it.
                // Additionally, the distance on an intersection should be zero.
                return (intersect, T::zero());
            }
            Closest::SinglePoint(pt) => {
                let dist = Haversine.distance(pt, *from);
                if dist < min_distance {
                    min_distance = dist;
                    rv = Closest::SinglePoint(pt);
                }
            }

            // If there is a case where we cannot figure out the closest,
            // Then it needs to be intereminate, instead of skipping
            // This however never happens for a Line/Point, which is the case here
            Closest::Indeterminate => return (Closest::Indeterminate, T::zero()),
        }
    }

    (rv, min_distance)
}

impl<T> HaversineClosestPoint<T> for Triangle<T>
where
    T: GeoFloat + FromPrimitive,
{
    fn haversine_closest_point(&self, from: &Point<T>) -> Closest<T> {
        if self.contains(from) {
            return Closest::Intersection(*from);
        }

        closest_closed_simple_poly(self.to_lines(), from).0
    }
}

impl<T> HaversineClosestPoint<T> for Rect<T>
where
    T: GeoFloat + FromPrimitive,
{
    fn haversine_closest_point(&self, from: &Point<T>) -> Closest<T> {
        if self.contains(from) {
            return Closest::Intersection(*from);
        }

        closest_closed_simple_poly(self.to_lines(), from).0
    }
}

impl<T> HaversineClosestPoint<T> for Polygon<T>
where
    T: GeoFloat + FromPrimitive,
{
    #[warn(unused_assignments)]
    fn haversine_closest_point(&self, from: &Point<T>) -> Closest<T> {
        if self.contains(from) {
            return Closest::Intersection(*from);
        }

        if self.exterior_coords_iter().count() < 3 {
            // Not really a polygon
            return Closest::Indeterminate;
        }

        let (mut rv, mut min_distance) = closest_closed_simple_poly(self.exterior().lines(), from);

        match rv {
            // Would not happen as it should be caught at the beginning of the function
            Closest::Intersection(_) => return rv,
            Closest::SinglePoint(_) => {}
            // Would not happen either. See other geometries for an explanation.
            // This is for future proof
            Closest::Indeterminate => return rv,
        }

        // Could be inside a inner ring
        for ls in self.interiors() {
            match closest_closed_simple_poly(ls.lines(), from) {
                // Would not happen as it should be caught at the beginning of the function
                (Closest::Intersection(pt), _) => return Closest::Intersection(pt),
                (Closest::SinglePoint(pt), dist) => {
                    if min_distance > dist {
                        min_distance = dist;
                        rv = Closest::SinglePoint(pt);
                    }
                }
                // Would not happen either.
                (Closest::Indeterminate, _) => unreachable!(),
            }
        }

        rv
    }
}

fn multi_geometry_nearest<G, I, T>(iter: I, from: &Point<T>) -> Closest<T>
where
    T: GeoFloat + FromPrimitive,
    G: HaversineClosestPoint<T>,
    I: IntoIterator<Item = G>,
{
    let mut min_distance = <T as num_traits::Float>::max_value();
    let mut rv = Closest::Indeterminate;

    for c in iter {
        match c.haversine_closest_point(from) {
            // This mean on top of the line.
            Closest::Intersection(pt) => return Closest::Intersection(pt),
            Closest::SinglePoint(pt) => {
                let dist = Haversine.distance(pt, *from);
                if dist < min_distance {
                    min_distance = dist;
                    rv = Closest::SinglePoint(pt);
                }
            }
            Closest::Indeterminate => return Closest::Indeterminate,
        }
    }
    rv
}

impl<T> HaversineClosestPoint<T> for MultiPoint<T>
where
    T: GeoFloat + FromPrimitive,
{
    fn haversine_closest_point(&self, from: &Point<T>) -> Closest<T> {
        multi_geometry_nearest(self, from)
    }
}

impl<T> HaversineClosestPoint<T> for MultiLineString<T>
where
    T: GeoFloat + FromPrimitive,
{
    fn haversine_closest_point(&self, from: &Point<T>) -> Closest<T> {
        multi_geometry_nearest(self, from)
    }
}

impl<T> HaversineClosestPoint<T> for MultiPolygon<T>
where
    T: GeoFloat + FromPrimitive,
{
    fn haversine_closest_point(&self, from: &Point<T>) -> Closest<T> {
        multi_geometry_nearest(self, from)
    }
}

impl<T> HaversineClosestPoint<T> for Geometry<T>
where
    T: GeoFloat + FromPrimitive,
{
    crate::geometry_delegate_impl! {
        fn haversine_closest_point(&self, from: &Point<T>) -> Closest<T>;
    }
}

impl<T> HaversineClosestPoint<T> for GeometryCollection<T>
where
    T: GeoFloat + FromPrimitive,
{
    fn haversine_closest_point(&self, from: &Point<T>) -> Closest<T> {
        multi_geometry_nearest(self, from)
    }
}

#[cfg(test)]
mod test {
    use wkt::TryFromWkt;

    use super::*;

    #[test]
    fn point_to_point() {
        let p_1 = Point::new(-84.74905, 32.61454);
        let p_2 = Point::new(-85.93942, 32.11055);

        if let Closest::SinglePoint(p) = p_1.haversine_closest_point(&p_2) {
            assert_relative_eq!(p_1, p);
        } else {
            panic!("Expecing Closest::SinglePoint");
        }

        if let Closest::SinglePoint(p) = p_2.haversine_closest_point(&p_1) {
            assert_relative_eq!(p_2, p);
        } else {
            panic!("Expecing Closest::SinglePoint");
        }

        if let Closest::Intersection(p) = p_2.haversine_closest_point(&p_2) {
            assert_relative_eq!(p_2, p);
        } else {
            panic!("Expecing Closest::Intersection");
        }
    }

    #[test]
    fn point_to_line_1() {
        let p_1 = Point::new(-84.74905, 32.61454);
        let p_2 = Point::new(-85.93942, 32.11055);
        let line = Line::new(p_2, p_1);

        let p_from = Point::new(-84.75625, 31.81056);
        if let Closest::SinglePoint(pt) = line.haversine_closest_point(&p_from) {
            assert_relative_eq!(pt, Point::new(-85.13337428852164, 32.45365659858937));
        } else {
            panic!("Did not get Closest::SinglePoint!");
        }

        let p_from = Point::new(-85.67211, 32.39774);
        if let Closest::SinglePoint(pt) = line.haversine_closest_point(&p_from) {
            assert_relative_eq!(pt, Point::new(-85.58999680564376, 32.26023534389268));
        } else {
            panic!("Did not get Closest::SinglePoint!");
        }
    }

    #[test]
    fn point_to_line_intersection() {
        let p_1 = Point::new(-84.74905, 32.61454);
        let p_2 = Point::new(-85.93942, 32.11055);
        let line = Line::new(p_2, p_1);

        if let Closest::Intersection(pt) = line.haversine_closest_point(&p_1) {
            assert!(pt == p_1);
        } else {
            panic!("Did not get Closest::Intersection!");
        }
    }

    #[test]
    fn point_to_line_intersection_2() {
        let p_1 = Point::new(-84.74905, 32.61454);
        let p_2 = Point::new(-85.93942, 32.11055);
        let line = Line::new(p_2, p_1);

        let p_from = Point::new(-85.13337428852164, 32.45365659858937);
        if let Closest::Intersection(pt) = line.haversine_closest_point(&p_from) {
            assert_relative_eq!(pt, p_from);
        } else {
            panic!("Did not get Closest::Intersection!");
        }
    }

    #[test]
    fn point_to_line_intersection_2_f32() {
        let p_1 = Point::new(-84.74905f32, 32.61454f32);
        let p_2 = Point::new(-85.93942f32, 32.11055f32);
        let line = Line::new(p_2, p_1);

        let p_from = Point::new(-85.13337f32, 32.453656f32);
        if let Closest::Intersection(pt) = line.haversine_closest_point(&p_from) {
            assert_relative_eq!(pt, p_from);
        } else {
            panic!("Did not get Closest::Intersection!");
        }
    }

    // Across the pole
    #[test]
    fn point_to_line_across_equator() {
        let p_1 = Point::new(-38.424_794_871_794_916, 75.137_388_461_538_48);
        let p_2 = Point::new(-28.608_712_820_512_863, -85.278_057_692_307_67);
        let line = Line::new(p_2, p_1);
        let p_from = Point::new(-25.86062, -87.32053);

        if let Closest::SinglePoint(pt) = line.haversine_closest_point(&p_from) {
            assert_relative_eq!(
                pt,
                Point::new(-28.608_712_820_512_864, -85.278_057_692_307_67)
            );
        } else {
            panic!("Did not get Closest::SinglePoint!");
        }
    }

    #[test]
    fn point_to_line_across_close_to_north_pole() {
        let p_1 = Point::new(-37.244_921_874_999_99, 79.508_612_500_000_03);
        let p_2 = Point::new(50.596_875_000_000_01, 81.054_628_125_000_02);
        let line = Line::new(p_2, p_1);
        let p_from = Point::new(8.15172, 77.40041);

        if let Closest::SinglePoint(pt) = line.haversine_closest_point(&p_from) {
            assert_relative_eq!(
                pt,
                Point::new(5.481_094_923_165_54, 82.998_280_987_615_33),
                epsilon = 1.0e-6
            );
        } else {
            panic!("Did not get Closest::SinglePoint!");
        }
    }

    #[test]
    fn point_to_linestring() {
        let wkt = "LineString (3.86503906250000284 11.71231367187503736, 9.48691406250000568 17.3341886718750402,
            13.28167968750000227 15.50707929687503395, 15.95207031249999829 9.18246992187503963,
            7.73007812500000568 8.33918867187503565, 16.0926171875000108 2.8578605468750311,
            23.26050781250000909 6.3715324218750311, 24.66597656250000625 14.24215742187503508,
            20.23875000000001023 13.6799699218750419, 19.11437500000000966 10.72848554687503508,
            18.20082031249999943 13.60969648437503565, 16.79535156250000227 17.54500898437503054,
            20.09820312500001194 17.26391523437503395, 22.27667968750000682 15.64762617187503224,
            24.24433593750001137 18.24774335937503622, 18.97382812500001137 18.38829023437503452)";

        let linestring = LineString::try_from_wkt_str(wkt).unwrap();

        let p_from = Point::new(17.02374, 10.57037);

        if let Closest::SinglePoint(pt) = linestring.haversine_closest_point(&p_from) {
            assert_relative_eq!(
                pt,
                Point::new(15.611386947136054, 10.006831648991811),
                epsilon = 1.0e-6
            );
        } else {
            panic!("Did not get Closest::SinglePoint!");
        }
    }

    #[test]
    fn point_to_empty_linestring() {
        let linestring = LineString::new(vec![]);

        let p_from = Point::new(17.02374, 10.57037);

        assert!(linestring.haversine_closest_point(&p_from) == Closest::Indeterminate);
    }

    #[test]
    fn point_to_poly_outside() {
        let wkt = "Polygon ((-10.99779296875000156 13.36373945312502087, -11.05049804687500092 13.85565351562501846,
            -10.21600097656250128 13.9171427734375186, -9.63624511718750121 14.47054609375001988,
            -8.2307763671875005 14.32121503906251903, -7.50168945312500135 13.65361738281252002, -7.50168945312500135 12.80155195312502059,
            -7.61588378906250085 13.50428632812501917, -7.76521484375000171 13.71510664062502016, -8.11658203125000099 13.87322187500002002,
            -8.27469726562500085 13.23197675781251981, -7.78278320312500149 12.7049259765625191, -8.25712890625000107 11.76501875000002073,
            -9.03892089843750135 11.91434980468751981, -10.33897949218750156 11.51906171875002016,
            -11.02414550781250213 12.46775312500001931, -9.0037841796875 12.33599042968752002, -8.46794921875000028 12.69614179687502009,
            -8.67876953125000128 13.39009199218751966, -8.44159667968750149 13.88200605468751903, -9.12676269531250028 14.10161054687501903,
            -9.68016601562500156 13.51307050781251995, -10.18964843750000071 13.02994062500001959, -10.99779296875000156 13.36373945312502087),
            (-8.59092773437500057 12.32720625000001924, -8.48551757812500185 12.0461125000000191,
                -8.16928710937500213 12.37112714843751959, -8.09022949218750043 12.74884687500001945, -8.59092773437500057 12.32720625000001924),
            (-10.42682128906250227 13.5569914062500203, -10.26870605468750242 13.38130781250001888,
                -9.8822021484375 13.58334394531252087, -9.84706542968750043 13.79416425781252009, -10.42682128906250227 13.5569914062500203))";

        let poly = Polygon::try_from_wkt_str(wkt).unwrap();

        let p_from = Point::new(-8.95108, 12.82790);

        if let Closest::SinglePoint(pt) = poly.haversine_closest_point(&p_from) {
            assert_relative_eq!(
                pt,
                Point::new(-8.732575801021413, 12.518536164563992),
                epsilon = 1.0e-6
            );
        } else {
            panic!("Did not get Closest::SinglePoint!");
        }
    }

    #[test]
    fn point_to_poly_outside_in_inner_ring() {
        let wkt = "Polygon ((-10.99779296875000156 13.36373945312502087, -11.05049804687500092 13.85565351562501846,
            -10.21600097656250128 13.9171427734375186, -9.63624511718750121 14.47054609375001988,
            -8.2307763671875005 14.32121503906251903, -7.50168945312500135 13.65361738281252002, -7.50168945312500135 12.80155195312502059,
            -7.61588378906250085 13.50428632812501917, -7.76521484375000171 13.71510664062502016, -8.11658203125000099 13.87322187500002002,
            -8.27469726562500085 13.23197675781251981, -7.78278320312500149 12.7049259765625191, -8.25712890625000107 11.76501875000002073,
            -9.03892089843750135 11.91434980468751981, -10.33897949218750156 11.51906171875002016,
            -11.02414550781250213 12.46775312500001931, -9.0037841796875 12.33599042968752002, -8.46794921875000028 12.69614179687502009,
            -8.67876953125000128 13.39009199218751966, -8.44159667968750149 13.88200605468751903, -9.12676269531250028 14.10161054687501903,
            -9.68016601562500156 13.51307050781251995, -10.18964843750000071 13.02994062500001959, -10.99779296875000156 13.36373945312502087),
            (-8.59092773437500057 12.32720625000001924, -8.48551757812500185 12.0461125000000191,
                -8.16928710937500213 12.37112714843751959, -8.09022949218750043 12.74884687500001945, -8.59092773437500057 12.32720625000001924),
            (-10.42682128906250227 13.5569914062500203, -10.26870605468750242 13.38130781250001888,
                -9.8822021484375 13.58334394531252087, -9.84706542968750043 13.79416425781252009, -10.42682128906250227 13.5569914062500203))";

        let poly = Polygon::try_from_wkt_str(wkt).unwrap();

        let p_from = Point::new(-8.38752, 12.29866);

        if let Closest::SinglePoint(pt) = poly.haversine_closest_point(&p_from) {
            assert_relative_eq!(
                pt,
                Point::new(-8.310007197809414, 12.226641293789331),
                epsilon = 1.0e-6
            );
        } else {
            panic!("Did not get Closest::SinglePoint!");
        }
    }

    #[test]
    fn point_to_poly_inside() {
        let wkt = "Polygon ((-10.99779296875000156 13.36373945312502087, -11.05049804687500092 13.85565351562501846,
            -10.21600097656250128 13.9171427734375186, -9.63624511718750121 14.47054609375001988,
            -8.2307763671875005 14.32121503906251903, -7.50168945312500135 13.65361738281252002, -7.50168945312500135 12.80155195312502059,
            -7.61588378906250085 13.50428632812501917, -7.76521484375000171 13.71510664062502016, -8.11658203125000099 13.87322187500002002,
            -8.27469726562500085 13.23197675781251981, -7.78278320312500149 12.7049259765625191, -8.25712890625000107 11.76501875000002073,
            -9.03892089843750135 11.91434980468751981, -10.33897949218750156 11.51906171875002016,
            -11.02414550781250213 12.46775312500001931, -9.0037841796875 12.33599042968752002, -8.46794921875000028 12.69614179687502009,
            -8.67876953125000128 13.39009199218751966, -8.44159667968750149 13.88200605468751903, -9.12676269531250028 14.10161054687501903,
            -9.68016601562500156 13.51307050781251995, -10.18964843750000071 13.02994062500001959, -10.99779296875000156 13.36373945312502087),
            (-8.59092773437500057 12.32720625000001924, -8.48551757812500185 12.0461125000000191,
                -8.16928710937500213 12.37112714843751959, -8.09022949218750043 12.74884687500001945, -8.59092773437500057 12.32720625000001924),
            (-10.42682128906250227 13.5569914062500203, -10.26870605468750242 13.38130781250001888,
                -9.8822021484375 13.58334394531252087, -9.84706542968750043 13.79416425781252009, -10.42682128906250227 13.5569914062500203))";

        let poly = Polygon::try_from_wkt_str(wkt).unwrap();

        let p_from = Point::new(-10.08341, 11.98792);

        if let Closest::Intersection(pt) = poly.haversine_closest_point(&p_from) {
            assert_relative_eq!(pt, p_from);
        } else {
            panic!("Did not get Closest::Intersection!");
        }
    }

    #[test]
    fn point_to_multi_polygon() {
        let wkt = "MultiPolygon (((-10.99779296875000156 13.36373945312502087, -11.05049804687500092 13.85565351562501846,
            -10.21600097656250128 13.9171427734375186, -9.63624511718750121 14.47054609375001988, -8.2307763671875005 14.32121503906251903,
            -7.50168945312500135 13.65361738281252002, -7.50168945312500135 12.80155195312502059,
            -7.61588378906250085 13.50428632812501917, -7.76521484375000171 13.71510664062502016, -8.11658203125000099 13.87322187500002002,
            -8.27469726562500085 13.23197675781251981, -7.78278320312500149 12.7049259765625191, -8.25712890625000107 11.76501875000002073,
            -9.03892089843750135 11.91434980468751981, -10.33897949218750156 11.51906171875002016,
            -11.02414550781250213 12.46775312500001931, -9.0037841796875 12.33599042968752002,
            -8.46794921875000028 12.69614179687502009, -8.67876953125000128 13.39009199218751966, -8.44159667968750149 13.88200605468751903,
            -9.12676269531250028 14.10161054687501903, -9.68016601562500156 13.51307050781251995,
            -10.18964843750000071 13.02994062500001959, -10.99779296875000156 13.36373945312502087),
            (-8.59092773437500057 12.32720625000001924, -8.48551757812500185 12.0461125000000191, -8.16928710937500213 12.37112714843751959,
                -8.09022949218750043 12.74884687500001945, -8.59092773437500057 12.32720625000001924),
            (-10.42682128906250227 13.5569914062500203, -10.26870605468750242 13.38130781250001888, -9.8822021484375 13.58334394531252087,
                -9.84706542968750043 13.79416425781252009, -10.42682128906250227 13.5569914062500203)),
            ((-8.99417648315430007 12.71261213378908828, -9.08641036987305029 12.51057600097658806,
                -8.83606124877929844 12.48861555175783877, -8.69990646362304787 12.6818675048828382,
                -8.74382736206055 12.77410139160158842, -8.86680587768555029 12.87951154785158892,
                -8.99417648315430007 12.71261213378908828)),((-8.99856857299804958 13.68326398925784027,
                -9.45095382690429986 13.16499738769534034, -9.48609054565430121 13.45926740722659076,
                -9.34993576049805064 13.7403611572265909, -8.91511886596680014 13.90726057128909154,
                -8.99856857299804958 13.68326398925784027)))";

        let poly = MultiPolygon::try_from_wkt_str(wkt).unwrap();

        let p_from = Point::new(-8.95108, 12.82790);

        if let Closest::SinglePoint(pt) = poly.haversine_closest_point(&p_from) {
            assert_relative_eq!(
                pt,
                Point::new(-8.922208260289914, 12.806949983368323),
                epsilon = 1.0e-6
            );
        } else {
            panic!("Did not get Closest::SinglePoint!");
        }
    }

    #[test]
    fn haversine_closest_point_intersecting_line() {
        // First a sensible case: the point to test just happens to be one of the points on the linestring
        let wkt = "LineString (3.86503906250000284 11.71231367187503736, 9.48691406250000568 17.3341886718750402)";
        let linestring = LineString::<f64>::try_from_wkt_str(wkt).unwrap();

        let line = linestring.lines().next().unwrap();

        let point = line.start_point();

        match linestring.haversine_closest_point(&point) {
            Closest::Intersection(_) => {
                // this is the correct answer
            }
            Closest::SinglePoint(_) => {
                panic!("unexpected: singlePoint")
            }
            Closest::Indeterminate => panic!("unexpected: indeterminate"),
        }

        // and now for the really degenerate case:
        // We have a linestring with overlapping coordinates, _and_ test that point
        let wkt = "LineString (3.86503906250000284 11.71231367187503736,
            3.86503906250000284 11.71231367187503736,
            9.48691406250000568 17.3341886718750402)";

        let linestring = LineString::<f64>::try_from_wkt_str(wkt).unwrap();
        let point = linestring.lines().next().unwrap().start_point();

        // because the overlapping point on the degenerate linestring is the exact same, we expect to get an intersection.
        if let Closest::Intersection(_) = linestring.haversine_closest_point(&point) {
            // this is correct
        } else {
            panic!("got wrong result")
        }
    }
}
