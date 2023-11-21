use std::ops::RangeInclusive;

use geo_types::*;

use crate::{ClosestPoint, Contains, CoordsIter, GeoFloat, LinesIter};

use super::types::{fold_closest, ClosestPointInfo, ClosestPointPreciseInfo, ConnectionKind};
use super::utils::{const_true, filter_points_not_creating_intersections, is_point_traversed_once};

/// Banana ranges contain index ranges of the parts of a banana. This handles all cases:
///
/// - banana through connection point
///   ┌─────────────┐
///   │     / \     │
///   │    /   \    │
///   │   /     \   │
///   │   ───────   │
///   │             │
///   │             │
///   │             │
///   └─────────────┘
/// - banana through connection line
///   ┌──────┬──────┐
///   │      │      │
///   │      │      │
///   │     / \     │
///   │    /   \    │
///   │   /     \   │
///   │   ───────   │
///   │             │
///   └─────────────┘
/// - recursive bananas (bananas in bananas)
#[derive(Debug)]
pub struct BananaRanges {
    /// range of indices, indexing into an exterior linestring
    ///
    /// the range points to one side of the banana
    pub min_range: RangeInclusive<usize>,
    /// range of indices, indexing into an exterior linestring
    ///
    /// the range points to the opposite side of the banana than the `min_range` field
    ///
    /// if this field here is none, it's assumed that every index not present in the first range is
    /// included in the second range
    pub max_range: Option<RangeInclusive<usize>>,
}

/// given
///
/// - a predicate for the index of the iterated point
/// - a predicate for the point itself
///
/// returns all coords in the polygons exterior that fulfill these predicates
fn filter_out_coords_by<F: GeoFloat>(
    index_predicate: impl Fn(&usize) -> bool,
    point_pred: impl Fn(Point<F>) -> bool,
) -> impl Fn(&LineString<F>) -> Vec<Coord<F>> {
    move |ls| {
        ls.points()
            .enumerate()
            .filter_map(|(i, p)| (index_predicate(&i) && point_pred(p)).then_some(p.0))
            .collect::<Vec<_>>()
    }
}

/// this function checks whether the polygon is a banana polygon and if so, it'll try to find a
/// minimal connecting line which either:
///
/// - connects a hole to the banana
/// - splits the banana (🍨🍌 yum) into two polygons, reducing the banananess*
///
/// * we can't prove that a split banana isn't banana anymore (it could be a whole bunch of bananas
/// in the same poly), but continuous splitting should eventually split the polygon into a polygon
/// without bananas
pub(crate) fn find_closest_lines_for_banana<F: GeoFloat>(
    p: &Polygon<F>,
) -> Option<ClosestPointPreciseInfo<F>> {
    // find banana ranges
    let banana_range = find_banana_ranges(p.exterior())?;

    let BananaRanges {
        min_range,
        max_range,
    } = banana_range;

    let lines = p.lines_iter().collect::<Vec<_>>();
    let traversed_once_pred = |p| is_point_traversed_once(&lines, p);
    let ext = p.exterior();

    // get valid coords based on banana ranges
    let inner_candidates =
        filter_out_coords_by(|i| min_range.contains(i), traversed_once_pred)(ext);
    let outer_exclusion_range = max_range.unwrap_or(min_range);
    let outer_candidates =
        filter_out_coords_by(|i| !outer_exclusion_range.contains(i), traversed_once_pred)(ext);

    // get best connection possible
    find_closest_lines_for_banana_inner(p, inner_candidates, outer_candidates)
}

pub(crate) fn find_and_split_outer_banana<F: GeoFloat>(p: &Polygon<F>) -> Option<MultiPolygon<F>> {
    // find banana ranges
    let banana_range = find_banana_ranges(p.exterior())?;

    let BananaRanges {
        min_range,
        max_range,
    } = banana_range;

    let ext = p.exterior();

    // get valid coords based on banana ranges
    let inner_candidates = filter_out_coords_by(|i| min_range.contains(i), const_true)(ext);
    let outer_exclusion_range = max_range.unwrap_or(min_range);
    let mut outer_candidates = filter_out_coords_by(
        |i| !outer_exclusion_range.contains(i) || i == outer_exclusion_range.start(),
        const_true,
    )(ext);
    outer_candidates.dedup();

    // check if neither polygon includes the other and if so, just return the split apart polys
    let poly1 = Polygon::new(LineString::new(inner_candidates), vec![]);
    let poly2 = Polygon::new(LineString::new(outer_candidates), vec![]);

    let is_outer_banana = !poly1.contains(&poly2) && !poly2.contains(&poly1);

    is_outer_banana.then(|| MultiPolygon::new(vec![poly1, poly2]))
}

/// for given source (start) and target (end) coords, find the smallest connection line between the
/// banana polygon and either:
///
/// - itself
/// - a interior hole
fn find_closest_lines_for_banana_inner<F: GeoFloat>(
    polygon: &Polygon<F>,
    source_coords: Vec<Coord<F>>,
    target_coords: Vec<Coord<F>>,
) -> Option<ClosestPointPreciseInfo<F>> {
    // Define some helping closures
    let lines = polygon
        .lines_iter()
        .filter(|l| l.start != l.end)
        .collect::<Vec<_>>();
    let target_linestring = LineString::new(target_coords);
    let iter_targets = || {
        std::iter::once(&target_linestring)
            .chain(polygon.interiors())
            .enumerate()
    };

    let find_closest_for = |point_in_self: Point<F>| {
        iter_targets()
            .map(|(id, ls)| {
                let ps = filter_points_not_creating_intersections(&lines, point_in_self)(ls);
                (id, ps)
            })
            .map(|(id, ps)| (id, ps.closest_point(&point_in_self)))
            .map(|(id, point_in_other)| ClosestPointInfo {
                point_in_other,
                point_in_self,
                from_linestring: ConnectionKind::from_banana_index(id),
            })
            .fold(None, fold_closest)
    };

    source_coords
        .into_iter()
        .map(Point::from)
        .filter_map(find_closest_for)
        .fold(None, fold_closest)
        .and_then(ClosestPointPreciseInfo::from_unprecise)
}

/// for one given banana polygon, traverses through the polygon and returns index ranges which mark
/// one side of the banana
fn find_banana_ranges<F: GeoFloat>(exterior: &LineString<F>) -> Option<BananaRanges> {
    let (_, ranges) = exterior.coords_iter().enumerate().fold(
        (vec![], vec![]),
        |(mut state_coords, mut ranges), (end, coord)| {
            // add new range if the current coord is closing some opened range
            ranges.extend(
                state_coords
                    .iter()
                    .position(|&other| other == coord)
                    .filter(|start| end - start + 1 < exterior.0.len())
                    .map(|start| start..=end),
            );
            // keep track of existing coords
            state_coords.push(coord);
            (state_coords, ranges)
        },
    );

    let range_len = |range: &&RangeInclusive<usize>| *range.end() - *range.start();
    let min_range = ranges.iter().min_by_key(range_len).cloned()?;

    // based on the min range, try to find the closest range that's bigger than the min range
    let max_range = get_max_range_from_min_range(exterior, &min_range);

    Some(BananaRanges {
        min_range,
        max_range,
    })
}

/// given some minimum banana range, this function calculates the banana range of the other side of
/// a possible banana connection linestring.
///
/// In case there is no banana linestring, this function returns None
fn get_max_range_from_min_range<F: GeoFloat>(
    exterior: &LineString<F>,
    min_range: &RangeInclusive<usize>,
) -> Option<RangeInclusive<usize>> {
    let start = *min_range.start();
    let end = *min_range.end();

    let ext_iter = || exterior.0.iter();
    let ext_iter_indexed = || ext_iter().enumerate();

    // get all points the are occuring more than once
    let duplicates = ext_iter()
        .filter(|c| ext_iter().skip(1).filter(|o| o == c).count() > 1)
        .collect::<Vec<_>>();

    // in case of a banana connection linestring, go along that linestring and find the point
    // that is furthest away from the min_range
    fn try_take_last_duplicate_in<'a, F: GeoFloat + 'a>(
        iter: impl Iterator<Item = (usize, &'a Coord<F>)>,
        duplicates: &[&Coord<F>],
    ) -> Option<usize> {
        iter.take_while(|(_, c)| duplicates.contains(c))
            .last()
            .map(|(i, _)| i)
    }

    let rev_iter_at_start = ext_iter_indexed().rev().skip(exterior.0.len() - start);
    let normal_iter_at_end = ext_iter_indexed().skip(end);

    // note that the early returns here make sure that we always have matching points on both
    // sides. This prevents us creating a range of duplicate points on the one side of the
    // connection line string while the other side actually features no duplicates
    //
    // in this case we just use the inverse min_range later on
    let max_start = try_take_last_duplicate_in(rev_iter_at_start, duplicates.as_slice())?;
    let max_end = try_take_last_duplicate_in(normal_iter_at_end, duplicates.as_slice())?;

    Some(max_start..=max_end)
}

/// simple predicate to check whether a polygon is a banana polygon
pub(crate) fn is_banana_polygon<F: GeoFloat>(poly: &Polygon<F>) -> bool {
    let ext = poly.exterior();
    let lines = ext.lines().collect::<Vec<_>>();
    ext.points().any(|p| !is_point_traversed_once(&lines, p))
}

#[cfg(test)]
mod banana_tests {
    use pretty_env_logger::env_logger::*;

    use super::*;

    fn gen_basic_inner_banana() -> Polygon {
        polygon!(
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 2.0, y: 0.0 },
            Coord { x: 2.0, y: 1.0 }, // banana start
            Coord { x: 1.0, y: 1.0 },
            Coord { x: 1.0, y: 3.0 },
            Coord { x: 3.0, y: 3.0 },
            Coord { x: 3.0, y: 1.0 },
            Coord { x: 2.0, y: 1.0 }, // banana end
            Coord { x: 2.0, y: 0.0 },
            Coord { x: 4.0, y: 0.0 },
            Coord { x: 4.0, y: 4.0 },
            Coord { x: 0.0, y: 4.0 },
        )
    }

    fn gen_long_connection_inner_banana() -> Polygon {
        polygon!(
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 2.0, y: 0.0 },
            Coord { x: 2.25, y: 0.25 },
            Coord { x: 1.75, y: 0.75 },
            Coord { x: 2.0, y: 1.0 }, // banana start
            Coord { x: 1.0, y: 1.0 },
            Coord { x: 1.0, y: 3.0 },
            Coord { x: 3.0, y: 3.0 },
            Coord { x: 3.0, y: 1.0 },
            Coord { x: 2.0, y: 1.0 }, // banana end
            Coord { x: 1.75, y: 0.75 },
            Coord { x: 2.25, y: 0.25 },
            Coord { x: 2.0, y: 0.0 },
            Coord { x: 4.0, y: 0.0 },
            Coord { x: 4.0, y: 4.0 },
            Coord { x: 0.0, y: 4.0 },
        )
    }

    fn gen_basic_outer_banana() -> Polygon {
        polygon!(
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 2.0, y: 0.0 },
            Coord { x: 2.0, y: -1.0 }, // banana start
            Coord { x: 1.0, y: -1.0 },
            Coord { x: 1.0, y: -3.0 },
            Coord { x: 3.0, y: -3.0 },
            Coord { x: 3.0, y: -1.0 },
            Coord { x: 2.0, y: -1.0 }, // banana end
            Coord { x: 2.0, y: 0.0 },
            Coord { x: 4.0, y: 0.0 },
            Coord { x: 4.0, y: 4.0 },
            Coord { x: 0.0, y: 4.0 },
        )
    }

    fn gen_long_connection_outer_banana() -> Polygon {
        polygon!(
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 2.0, y: 0.0 },
            Coord { x: 2.25, y: -0.25 },
            Coord { x: 1.75, y: -0.75 },
            Coord { x: 2.0, y: -1.0 }, // banana start
            Coord { x: 1.0, y: -1.0 },
            Coord { x: 1.0, y: -3.0 },
            Coord { x: 3.0, y: -3.0 },
            Coord { x: 3.0, y: -1.0 },
            Coord { x: 2.0, y: -1.0 }, // banana end
            Coord { x: 1.75, y: -0.75 },
            Coord { x: 2.25, y: -0.25 },
            Coord { x: 2.0, y: 0.0 },
            Coord { x: 4.0, y: 0.0 },
            Coord { x: 4.0, y: 4.0 },
            Coord { x: 0.0, y: 4.0 },
        )
    }

    fn gen_recursive_banana() -> Polygon {
        polygon!(
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 2.0, y: 0.0 },
            Coord { x: 2.0, y: 1.0 }, // level 1 banana start
            Coord { x: 1.0, y: 1.0 },
            Coord { x: 1.0, y: 3.0 },
            Coord { x: 3.0, y: 3.0 },
            Coord { x: 3.0, y: 2.0 },
            Coord { x: 4.0, y: 2.0 }, // level 2 banana start
            Coord { x: 4.0, y: 3.0 },
            Coord { x: 6.0, y: 3.0 },
            Coord { x: 6.0, y: 1.0 },
            Coord { x: 4.0, y: 1.0 },
            Coord { x: 4.0, y: 2.0 }, // level 2 banana end
            Coord { x: 3.0, y: 2.0 },
            Coord { x: 3.0, y: 1.0 },
            Coord { x: 2.0, y: 1.0 }, // level 1 banana end
            Coord { x: 2.0, y: 0.0 },
            Coord { x: 10.0, y: 0.0 },
            Coord { x: 10.0, y: 10.0 },
            Coord { x: 0.0, y: 10.0 },
        )
    }

    fn run_finder_test_for<F: GeoFloat>(
        gen: impl Fn() -> Polygon<F>,
        expected_inner_range: RangeInclusive<usize>,
        expected_outer_range: Option<RangeInclusive<usize>>,
    ) {
        _ = try_init();

        let banana = gen();
        let banana_connection = find_banana_ranges(banana.exterior());

        assert!(banana_connection.is_some());
        let banana_connection = banana_connection.unwrap();

        assert_eq!(banana_connection.min_range, expected_inner_range);
        assert_eq!(
            banana.exterior().0[*banana_connection.min_range.start()],
            banana.exterior().0[*banana_connection.min_range.end()]
        );
        assert_eq!(banana_connection.max_range, expected_outer_range);
        assert_eq!(
            banana_connection
                .max_range
                .as_ref()
                .map(|range| banana.exterior().0[*range.start()]),
            banana_connection
                .max_range
                .as_ref()
                .map(|range| banana.exterior().0[*range.end()]),
        );
    }

    mod found_banana {
        use super::*;

        #[test]
        fn basic_inner_banana_found() {
            run_finder_test_for(gen_basic_inner_banana, 2..=7, Some(1..=8));
        }

        #[test]
        fn long_inner_banana_found() {
            run_finder_test_for(gen_long_connection_inner_banana, 4..=9, Some(1..=12));
        }

        #[test]
        fn basic_outer_banana_found() {
            run_finder_test_for(gen_basic_outer_banana, 2..=7, Some(1..=8));
        }

        #[test]
        fn long_outer_banana_found() {
            run_finder_test_for(gen_long_connection_outer_banana, 4..=9, Some(1..=12));
        }

        #[test]
        fn recursive_banana_found() {
            run_finder_test_for(gen_recursive_banana, 7..=12, Some(6..=13));
        }
    }

    fn run_fixing_test_inner<F: GeoFloat>(
        gen: impl Fn() -> Polygon<F>,
        expected_closest: Option<ClosestPointPreciseInfo<F>>,
    ) {
        _ = try_init();
        let banana = gen();
        let fixed = find_closest_lines_for_banana(&banana);

        assert_eq!(fixed, expected_closest);
    }

    fn run_fixing_test_outer<F: GeoFloat>(
        gen: impl Fn() -> Polygon<F>,
        expected_closest: Option<MultiPolygon<F>>,
    ) {
        _ = try_init();
        let banana = gen();
        let fixed = find_and_split_outer_banana(&banana);

        assert_eq!(fixed, expected_closest);
    }

    #[test]
    fn basic_inner_banana_fixed() {
        run_fixing_test_inner(
            gen_basic_inner_banana,
            Some(ClosestPointPreciseInfo {
                from_linestring: ConnectionKind::Exterior,
                point_in_self: Point::new(1.0, 1.0),
                point_in_other: Point::new(0.0, 0.0),
            }),
        );
    }

    #[test]
    fn long_inner_banana_fixed() {
        run_fixing_test_inner(
            gen_long_connection_inner_banana,
            Some(ClosestPointPreciseInfo {
                from_linestring: ConnectionKind::Exterior,
                point_in_self: Point::new(1.0, 1.0),
                point_in_other: Point::new(0.0, 0.0),
            }),
        );
    }

    #[test]
    fn basic_outer_banana_fixed() {
        let expected = [
            vec![
                Coord { x: 2.0, y: -1.0 },
                Coord { x: 1.0, y: -1.0 },
                Coord { x: 1.0, y: -3.0 },
                Coord { x: 3.0, y: -3.0 },
                Coord { x: 3.0, y: -1.0 },
            ],
            vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 2.0, y: 0.0 },
                Coord { x: 4.0, y: 0.0 },
                Coord { x: 4.0, y: 4.0 },
                Coord { x: 0.0, y: 4.0 },
            ],
        ]
        .map(|coords| Polygon::new(LineString::new(coords), vec![]))
        .to_vec();
        let expected = MultiPolygon::new(expected);
        run_fixing_test_outer(gen_basic_outer_banana, Some(expected));
    }

    #[test]
    fn long_outer_banana_fixed() {
        let expected = [
            vec![
                Coord { x: 2.0, y: -1.0 },
                Coord { x: 1.0, y: -1.0 },
                Coord { x: 1.0, y: -3.0 },
                Coord { x: 3.0, y: -3.0 },
                Coord { x: 3.0, y: -1.0 },
            ],
            vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 2.0, y: 0.0 },
                Coord { x: 4.0, y: 0.0 },
                Coord { x: 4.0, y: 4.0 },
                Coord { x: 0.0, y: 4.0 },
            ],
        ]
        .map(|coords| Polygon::new(LineString::new(coords), vec![]))
        .to_vec();
        let expected = MultiPolygon::new(expected);
        run_fixing_test_outer(gen_long_connection_outer_banana, Some(expected));
    }

    #[test]
    fn real1() {
        let real = || {
            polygon! {
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 10.0, y: 0.0 },
                Coord { x: 10.0, y: 3.0 },
                Coord { x: 0.0, y: 3.0 },
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 4.0, y: 1.0 },
                Coord { x: 4.0, y: 2.0 },
                Coord { x: 4.25, y: 2.0 },
                Coord { x: 4.75, y: 2.0 },
                Coord { x: 5.0, y: 2.0 },
                Coord { x: 5.0, y: 1.0 },
                Coord { x: 4.75, y: 1.0 },
                Coord { x: 4.75, y: 2.0 },
                Coord { x: 4.25, y: 2.0 },
                Coord { x: 4.25, y: 1.0 },
                Coord { x: 4.0, y: 1.0 },
                Coord { x: 0.0, y: 0.0 },
            }
        };
        run_fixing_test_inner(
            real,
            Some(ClosestPointPreciseInfo {
                from_linestring: ConnectionKind::Exterior,
                point_in_self: Point::new(0.0, 3.0),
                point_in_other: Point::new(4.0, 2.0),
            }),
        );
    }

    #[test]
    fn real2() {
        let real = || {
            polygon! {
                Coord { x: 4.0, y: 2.0 },
                Coord { x: 4.25, y: 2.0 },
                Coord { x: 4.75, y: 2.0 },
                Coord { x: 5.0, y: 2.0 },
                Coord { x: 5.0, y: 1.0 },
                Coord { x: 4.75, y: 1.0 },
                Coord { x: 4.75, y: 2.0 },
                Coord { x: 4.25, y: 2.0 },
                Coord { x: 4.25, y: 1.0 },
                Coord { x: 4.0, y: 1.0 },
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 10.0, y: 0.0 },
                Coord { x: 10.0, y: 3.0 },
                Coord { x: 0.0, y: 3.0 },
                Coord { x: 4.0, y: 2.0 },
            }
        };
        run_fixing_test_inner(
            real,
            Some(ClosestPointPreciseInfo {
                from_linestring: ConnectionKind::Exterior,
                point_in_self: Point::new(4.75, 1.0),
                point_in_other: Point::new(4.25, 1.0),
            }),
        );
    }
}
