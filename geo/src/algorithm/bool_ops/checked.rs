use crate::MultiPolygon;
use crate::OpType;

pub(super) mod sealed {
    use crate::bool_ops::checked::CheckedBooleanOps;
    use crate::BooleanOps;
    use crate::Contains;
    use crate::GeoFloat;
    use std::f64::consts::PI;

    use geo_types::Coord;
    use geo_types::Line;
    use geo_types::LineString;
    use geo_types::MultiPolygon;
    use geo_types::Point;
    use geo_types::Polygon;

    /// The preprocess trait for checked boolean operations
    ///
    /// # The Problem
    ///
    /// The goal of the implemented preprocess function is to fix problems with the current
    /// implementation of the boolean operations algorithm. It had notable difficulties in some
    /// situations. If
    ///
    /// - `boolop(A,B)` was called
    /// - `B` had some points in its geometry which are located near or directly on lines of geometry A
    ///
    /// then in on cases the boolop algo could panic in an unrecoverable way.
    ///
    /// # Related issues
    ///
    /// - https://github.com/georust/geo/issues/913
    /// - https://github.com/georust/geo/issues/976
    /// - https://github.com/georust/geo/issues/1053
    /// - https://github.com/georust/geo/issues/1064
    ///
    /// # The Solution
    ///
    /// The preprocess trait simply fixes the problem mentioned above by slightly adjusting each point
    /// that might cause trouble. These points are slightly relocated to a place which is considered
    /// "checked". This, of course, causes some inaccuracies which turned out to be negligible in all
    /// currently conducted test cases
    ///
    /// # Limitations
    ///
    /// Some observed limitations are:
    ///
    /// - The preprocessing will probably have non trivial performance implications and you should test
    /// whether it's feasible for you to use this "fix"
    /// - It seems that if the scales of the arguments `A` and `B` are orders of magnitude apart, the
    /// new checked implementation can potentially fail to find intersection (possibly also other ops).
    /// This was observed in the test case 1064 and it's not clear how the "unchecked" algo would respond
    /// in similar cases
    /// - The algorithm only works on a specific scale around `0.0`. If the coordinate values grow too
    /// big or too small then this algorithm can still run into issues. It's recommended to use the
    /// `Scale` trait to scale arguments to reasonable sizes around `0.0` and then scale the result
    /// back up/down again
    pub trait PreprocessBoolops: BooleanOps {
        fn preprocess(&self, other: &Self) -> Self;
    }

    impl<T: GeoFloat> PreprocessBoolops for Polygon<T> {
        fn preprocess(&self, other: &Self) -> Self {
            let const_lines = create_line_hitboxes(self).collect::<Vec<_>>();
            adjust_other(&const_lines, other)
        }
    }

    impl<T: GeoFloat> PreprocessBoolops for MultiPolygon<T> {
        fn preprocess(&self, other: &Self) -> Self {
            let const_lines = self
                .iter()
                .flat_map(|p| create_line_hitboxes(p))
                .collect::<Vec<_>>();

            let others = other
                .iter()
                .map(|other| adjust_other(&const_lines, other))
                .collect::<Vec<_>>();

            MultiPolygon::new(others)
        }
    }

    // creates hitboxes for each line of the given geometry and returns the hitbox polygon together
    // with the respective line
    fn create_line_hitboxes<T: GeoFloat>(
        poly: &Polygon<T>,
    ) -> impl Iterator<Item = (Polygon<T>, Line<T>)> + '_ {
        // The line hitbox is basically a line with some epsilon padding applied. The value of
        // epsilon was carefully chosen to make all known test cases work. It might be of need to
        // change it again in the future. Here's a picture of the hitbox:
        //
        // ┌───▲───┐
        // │   │   │
        // │   │   │
        // │   │   │
        // │   │   │
        // └───┴───┘
        //  eps eps
        const MAGIC_NUMBER: i32 = 13;
        let eps = T::epsilon() * (T::one() + T::one()).powi(MAGIC_NUMBER);
        poly.exterior()
            .lines()
            .chain(poly.interiors().iter().flat_map(|ls| ls.lines()))
            .map(move |line| {
                let perp = {
                    let dir = line.delta();
                    geo_types::coord! {x: -dir.y, y: dir.x}
                };
                let ext = geo_types::LineString::new(vec![
                    line.start + perp * eps,
                    line.end + perp * eps,
                    line.end - perp * eps,
                    line.start - perp * eps,
                ]);
                (Polygon::new(ext, vec![]), line)
            })
    }

    fn adjust_other<T: GeoFloat>(
        const_lines: &[(Polygon<T>, Line<T>)],
        other: &Polygon<T>,
    ) -> Polygon<T> {
        let ext = LineString::new(
            other
                .exterior()
                .points()
                .map(|point| apply_min_distance(const_lines, point))
                .collect::<Vec<_>>(),
        );

        let ints = other
            .interiors()
            .iter()
            .map(|ls| {
                LineString::new(
                    ls.points()
                        .map(|point| apply_min_distance(const_lines, point))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();

        Polygon::new(ext, ints)
    }

    // checks if a point is located within on of the hitboxes and if so, it relocated
    // the point. Otherwise it is left unchanged
    fn apply_min_distance<T: GeoFloat>(
        const_lines: &[(Polygon<T>, Line<T>)],
        point: Point<T>,
    ) -> Coord<T> {
        const_lines
            .iter()
            .any(|(hitbox, line)| {
                point != line.start_point() && point != line.end_point() && hitbox.contains(&point)
            })
            .then(|| relocate_point_outside_hitboxes(const_lines, &point))
            .unwrap_or(point.0)
    }

    // relocates a point to a place outside of all of the given hitboxes
    fn relocate_point_outside_hitboxes<T: GeoFloat>(
        const_lines: &[(Polygon<T>, Line<T>)],
        center: &Point<T>,
    ) -> Coord<T> {
        // This is an iterator that spirals around the given center coordinate in 1/16th turns and
        // with increasing radius of T::epsilon every turn
        let mut spiral_iter = (0..)
            .map(|i| T::from(i).unwrap())
            .map(|i: T| {
                let phi: T = i * T::from(PI * 0.125).unwrap();
                let r: T = i * T::epsilon();
                let x = phi.sin() * r;
                let y = phi.cos() * r;
                geo_types::Coord { x, y }
            })
            .map(|offset| offset + center.0);
        // find the first point in the iterator that is not located in any of the hitboxes
        spiral_iter
            .find(|p| !const_lines.iter().any(|(hitbox, _)| hitbox.contains(p)))
            .unwrap()
    }

    impl<T: PreprocessBoolops> CheckedBooleanOps for T {}
}

/// Boolean operations are set operations on geometries considered as a subset
/// of the 2-D plane. The operations supported are: intersection, union, xor or
/// symmetric difference, and set-difference on pairs of 2-D geometries and
/// clipping a 1-D geometry with self.
///
/// These operations are implemented on [`Polygon`] and the [`MultiPolygon`]
/// geometries.
///
/// # Validity
///
/// Note that the operations are strictly well-defined only on *valid*
/// geometries. However, the implementation generally works well as long as the
/// interiors of polygons are contained within their corresponding exteriors.
///
/// Degenerate 2-d geoms with 0 area are handled, and ignored by the algorithm.
/// In particular, taking `union` with an empty geom should remove degeneracies
/// and fix invalid polygons as long the interior-exterior requirement above is
/// satisfied.
pub trait CheckedBooleanOps: sealed::PreprocessBoolops {
    fn checked_boolean_op(
        &self,
        other: &Self,
        op: OpType,
    ) -> Result<MultiPolygon<Self::Scalar>, CheckedBoolopsError> {
        let checked_other = self.preprocess(other);
        Ok(self.boolean_op(&checked_other, op))
    }
    fn checked_intersection(
        &self,
        other: &Self,
    ) -> Result<MultiPolygon<Self::Scalar>, CheckedBoolopsError> {
        self.checked_boolean_op(other, OpType::Intersection)
    }
    fn checked_union(
        &self,
        other: &Self,
    ) -> Result<MultiPolygon<Self::Scalar>, CheckedBoolopsError> {
        self.checked_boolean_op(other, OpType::Union)
    }
    fn checked_xor(&self, other: &Self) -> Result<MultiPolygon<Self::Scalar>, CheckedBoolopsError> {
        self.checked_boolean_op(other, OpType::Xor)
    }
    fn checked_difference(
        &self,
        other: &Self,
    ) -> Result<MultiPolygon<Self::Scalar>, CheckedBoolopsError> {
        self.checked_boolean_op(other, OpType::Difference)
    }
}

// Placeholder
#[derive(Debug)]
pub enum CheckedBoolopsError {}

#[cfg(test)]
mod test {
    use geo_svg::{Color, ToSvg};
    use wkt::TryFromWkt;

    use crate::bool_ops::checked::CheckedBooleanOps;
    use crate::*;

    fn info_log_svgs(a: &impl ToSvg, b: &impl ToSvg, c: &impl ToSvg) {
        let svg = a
            .to_svg()
            .with_color(Color::Named("#ff000044"))
            .and(b.to_svg().with_color(Color::Named("#0000ff44")))
            .and(c.to_svg().with_color(Color::Named("#88888888")));
        info!("\n\n\n{svg}\n\n\n");
    }

    #[test]
    fn issue_1053_doesnt_panic() {
        _ = pretty_env_logger::try_init();
        let geo1 = Polygon::<f32>::new(
            LineString(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 0.0, y: 200.0 },
                Coord { x: 200.0, y: 200.0 },
                Coord { x: 200.0, y: 0.0 },
                Coord { x: 0.0, y: 0.0 },
            ]),
            vec![],
        );
        let geo2 = Polygon::<f32>::new(
            LineString(vec![
                Coord {
                    x: -0.17588139,
                    y: 0.0015348792,
                },
                Coord {
                    x: 1.5845897,
                    y: 201.73154,
                },
                Coord {
                    x: 200.1759,
                    y: 199.99846,
                },
                Coord {
                    x: 198.41544,
                    y: -1.7315454,
                },
                Coord {
                    x: -0.17588139,
                    y: 0.0015348792,
                },
            ]),
            vec![],
        );
        let intersection = geo1.checked_intersection(&geo2).unwrap();
        info_log_svgs(&geo1, &geo2, &intersection);
    }

    #[test]
    fn issue_1064_doesnt_panic() {
        _ = pretty_env_logger::try_init();
        let a: Polygon = Polygon::try_from_wkt_str("POLYGON ((709799.9999999999 4535330.115932672, 709800.0000000001 4535889.8772568945, 709800.0057476197 4535889.994252375, 709800.1227431173 4535890.000000001, 710109.8788852151 4535889.999999996, 710109.994510683 4535889.99439513, 710110.0025226776 4535889.878911494, 710119.9974094903 4535410.124344491, 710119.9939843 4535410.005891683, 710119.8756285358 4535410.000000003, 710050.1240139506 4535410.000000003, 710050.0040320279 4535410.005955433, 710049.9539423736 4535410.115144073, 710038.1683017325 4535439.579245671, 710037.9601922985 4535440.0325333765, 710037.7079566419 4535440.462831489, 710037.4141048047 4535440.865858022, 710037.0815609565 4535441.237602393, 710036.7136342974 4535441.57436531, 710036.3139861295 4535441.872795589, 710035.8865934211 4535442.129923492, 710035.4357092284 4535442.343190307, 710034.9658203793 4535442.510473766, 710034.4816028172 4535442.6301092245, 710033.9878750739 4535442.7009061435, 710033.489550319 4535442.722160025, 710032.9915874689 4535442.6936593605, 710032.4989418354 4535442.615687774, 710032.016515821 4535442.48902117, 710031.54911013 4535442.314920021, 710031.1013759954 4535442.095116852, 710030.6777688961 4535441.831798956, 710030.2825042206 4535441.527586658, 710029.9195153153 4535441.185507218, 710029.5924143443 4535440.808964732, 710029.3044563448 4535440.401706239, 710029.0585068383 4535439.967784446, 710028.8570133082 4535439.511517367, 710028.701980853 4535439.037445406, 710028.5949522265 4535438.550286128, 710028.536992489 4535438.05488734, 710020.008429941 4535310.126449097, 710019.9938185212 4535310.0024022255, 710019.9208325277 4535309.901040657, 709980.0772727348 4535260.096590921, 709979.9987806884 4535260.007853539, 709979.8970781134 4535260.068614591, 709920.1022372885 4535299.931841814, 709920.0058878824 4535300.003151096, 709920.0000000001 4535300.122873942, 709920.0000000002 4535324.451138855, 709919.9762868701 4535324.937522437, 709919.9053724057 4535325.419292543, 709919.7879292492 4535325.891879465, 709919.6250713767 4535326.350800604, 709919.4183435373 4535326.7917029755, 709919.1697065973 4535327.210404501, 709918.8815189389 4535327.602933702, 709918.5565140966 4535327.965567328, 709918.1977748218 4535328.294865718, 709917.8087038476 4535328.5877053905, 709917.3929916087 4535328.841308688, 709916.9545812428 4535329.053270123, 709916.497631181 4535329.221579175, 709916.0264757108 4535329.344639407, 709915.5455838591 4535329.421283546, 709915.059517008 4535329.450784619, 709914.5728856224 4535329.43286279, 709914.0903055237 4535329.367688051, 709913.6163541072 4535329.255878603, 709913.1555269207 4535329.098494996, 709912.7121950255 4535328.89703004, 709912.2905635366 4535328.653394689, 709911.8946317348 4535328.369899881, 709911.5281551343 4535328.049234638, 709911.1946098561 4535327.694440548, 709910.8971596619 4535327.308882924, 709910.638625942 4535326.896218885, 709910.4214609532 4535326.460362643, 709910.2477245614 4535326.005448406, 709910.119064699 4535325.53579115, 709900.0266965557 4535280.120134492, 709899.9954816136 4535280.006411615, 709899.8778852832 4535280.015264336, 709820.1219250998 4535289.984759362, 709820.0038570677 4535290.005451573, 709819.9450491015 4535290.109901799, 709800.0518466672 4535329.896306664, 709800.0053207672 4535330.001256061, 709799.9999999999 4535330.115932672))").unwrap();
        let b: Polygon = Polygon::try_from_wkt_str("POLYGON ((709800 4600020, 809759.9999999993 4600020, 809759.9999999987 4500000, 709800.0000000003 4500000, 709800 4590240, 709800 4600020))").unwrap();

        // NOTE: The scale here is important. The checked fix only works on a specific scale of
        // polygons. It's possible to scale down and up again as a way to preserve scaling
        let intersection = a
            .scale(0.01)
            .checked_intersection(&b.scale(0.01))
            .unwrap()
            .scale(0.01_f64.recip());
        info_log_svgs(&a, &b, &intersection);
    }

    #[test]
    fn issue_913_doesnt_panic_1() {
        _ = pretty_env_logger::try_init();
        let p1: MultiPolygon<f32> = MultiPolygon::new(vec![Polygon::new(
            LineString::from(vec![
                Coord {
                    x: 140.50,
                    y: 62.24,
                },
                Coord {
                    x: 140.07,
                    y: 62.34,
                },
                Coord {
                    x: 136.94,
                    y: 63.05,
                },
                Coord {
                    x: 140.50,
                    y: 62.24,
                },
            ]),
            vec![],
        )]);
        let p2: MultiPolygon<f32> = MultiPolygon::new(vec![Polygon::new(
            LineString::from(vec![
                Coord {
                    x: 142.50,
                    y: 61.44,
                },
                Coord {
                    x: 142.06,
                    y: 61.55,
                },
                Coord {
                    x: 140.07,
                    y: 62.34,
                },
                Coord {
                    x: 142.50,
                    y: 61.44,
                },
            ]),
            vec![],
        )]);

        let intersection = p2.checked_intersection(&p1).unwrap();
        info_log_svgs(&p1, &p2, &intersection);
        let difference = p1.checked_difference(&intersection).unwrap();
        info_log_svgs(&p1, &intersection, &difference);
    }

    #[test]
    fn issue_913_doesnt_panic_2() {
        _ = pretty_env_logger::try_init();
        let pg1 = Polygon::new(
            LineString(vec![
                Coord {
                    x: 367.679,
                    y: 302.785,
                },
                Coord {
                    x: 146.985,
                    y: 90.13,
                },
                Coord {
                    x: 235.63,
                    y: 197.771,
                },
                Coord {
                    x: 222.764,
                    y: 233.747,
                },
                Coord {
                    x: 200.38180602615253,
                    y: 288.3378373783007,
                },
                Coord {
                    x: 207.575,
                    y: 288.205,
                },
                Coord {
                    x: 367.679,
                    y: 302.785,
                },
            ]),
            vec![],
        );

        let pg2 = Polygon::new(
            LineString(vec![
                Coord {
                    x: 367.679,
                    y: 302.785,
                },
                Coord {
                    x: 203.351,
                    y: 279.952,
                },
                Coord {
                    x: 200.373,
                    y: 288.338,
                },
                Coord {
                    x: 206.73187706275607,
                    y: 288.2205700292493,
                },
                Coord {
                    x: 208.489,
                    y: 286.233,
                },
                Coord {
                    x: 212.24,
                    y: 285.478,
                },
                Coord {
                    x: 367.679,
                    y: 302.785,
                },
            ]),
            vec![],
        );

        let union = pg1.checked_union(&pg2).unwrap();
        info_log_svgs(&pg1, &pg2, &union)
    }

    #[test]
    fn issue_913_doesnt_panic_3() {
        _ = pretty_env_logger::try_init();
        let p1: MultiPolygon<f32> = MultiPolygon::new(vec![Polygon::new(
            LineString::from(vec![
                Coord {
                    x: 142.50143433,
                    y: 61.44324493,
                },
                Coord {
                    x: 142.05662537,
                    y: 61.54975128,
                },
                Coord {
                    x: 140.07255554,
                    y: 62.33781433,
                },
                Coord {
                    x: 142.50143433,
                    y: 61.44324493,
                },
            ]),
            vec![],
        )]);
        let p2: MultiPolygon<f32> = MultiPolygon::new(vec![Polygon::new(
            LineString::from(vec![
                Coord {
                    x: 140.50358582,
                    y: 62.23768616,
                },
                Coord {
                    x: 140.07255554,
                    y: 62.33781433,
                },
                Coord {
                    x: 136.93997192,
                    y: 63.05339432,
                },
                Coord {
                    x: 140.50358582,
                    y: 62.23768616,
                },
            ]),
            vec![],
        )]);

        let intersection = p1.checked_intersection(&p2).unwrap();
        info_log_svgs(&p1, &p2, &intersection);
        let difference = p1.checked_difference(&intersection).unwrap();
        info_log_svgs(&p1, &p2, &difference);
    }

    #[test]
    fn issue_913_doesnt_panic_4() {
        _ = pretty_env_logger::try_init();

        let poly1: Polygon = Polygon::try_from_wkt_str("POLYGON((204.0 287.0,206.69670020700084 288.2213844497616,200.38308697914755 288.338793163584,204.0 287.0))").unwrap();
        let poly2: Polygon = Polygon::try_from_wkt_str("POLYGON((210.0 290.0,204.07584923592933 288.2701221108328,212.24082541367974 285.47846008552216,210.0 290.0))").unwrap();

        let union = poly1.checked_union(&poly2).unwrap();
        info_log_svgs(&poly1, &poly2, &union);
    }

    #[test]
    fn issue_976_doesnt_panic_1() {
        _ = pretty_env_logger::try_init();

        let geo1 = Polygon::new(
            LineString(vec![
                Coord { x: -1.0, y: 46.0 },
                Coord { x: 8.0, y: 46.0 },
                Coord { x: 8.0, y: 39.0 },
                Coord { x: -1.0, y: 39.0 },
                Coord { x: -1.0, y: 46.0 },
            ]),
            vec![LineString(vec![
                Coord { x: 2.0, y: 45.0 },
                Coord { x: 7.0, y: 45.0 },
                Coord { x: 7.0, y: 44.0 },
                Coord { x: 5.0, y: 42.0 },
                Coord { x: 5.0, y: 41.0 },
                Coord { x: 0.0, y: 43.0 },
                Coord { x: 2.0, y: 45.0 },
            ])],
        );
        let geo2 = Polygon::new(
            LineString(vec![
                Coord { x: 0.0, y: 42.0 },
                Coord { x: 6.0, y: 44.0 },
                Coord { x: 4.0, y: 40.0 },
                Coord { x: 0.0, y: 42.0 },
            ]),
            vec![],
        );
        let mut left = MultiPolygon::new(vec![geo1]);
        let mut right = MultiPolygon::new(vec![geo2]);
        let shift = |c: Coord| Coord {
            x: c.x + 931230.,
            y: c.y + 412600.,
        };
        left.map_coords_in_place(shift);
        right.map_coords_in_place(shift);
        let difference = left.checked_difference(&right).unwrap();
        info_log_svgs(&left, &right, &difference);
    }
}
