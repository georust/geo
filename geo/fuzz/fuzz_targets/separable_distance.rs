#![no_main]

// Cross-check the separable distance fast path against a brute-force segment sweep.
//
// `Euclidean::distance` takes a project-and-prune fast path when two geometries have
// bounding boxes that are separated along an axis (see `separable_geometry_distance_fast`
// in `geo`). The pruning logic there is subtle: an unsound prefix skip once caused it to
// return distances that were too large. This target generates two arbitrary line strings,
// translates the second so that the pair is always x-separated, and asserts that the fast
// path agrees with the minimum over all segment pairs.
//
// Coordinates are restricted to zero or magnitudes in `[1e-6, 1e6]`. Outside that range
// the segment-distance primitive that both sides share loses precision (near-degenerate
// segments underflow in the projection arithmetic), and because the two sides evaluate
// different segment pairs, that noise surfaces as spurious disagreement.

use geo::{Distance, Euclidean};
use geo_types::{LineString, Polygon};
use libfuzzer_sys::fuzz_target;

const COORD_MIN: f64 = 1e-6;
const COORD_MAX: f64 = 1e6;

/// Minimum distance between two line strings, by brute force over every segment pair.
fn brute_force_line_string_distance(a: &LineString<f64>, b: &LineString<f64>) -> f64 {
    a.lines()
        .flat_map(|seg_a| {
            b.lines()
                .map(move |seg_b| Euclidean.distance(&seg_a, &seg_b))
        })
        .fold(f64::INFINITY, f64::min)
}

fn in_bounds(line_string: &LineString<f64>) -> bool {
    fn ok(value: f64) -> bool {
        value == 0.0 || (COORD_MIN..=COORD_MAX).contains(&value.abs())
    }
    line_string.coords().all(|c| ok(c.x) && ok(c.y))
}

fn assert_close(actual: f64, expected: f64, context: &str) {
    // Both sides evaluate candidate pairs with the same segment primitive, so they agree
    // to within floating-point noise unless the pruning is unsound
    let tolerance = 1e-9 * (1.0 + expected);
    assert!(
        (actual - expected).abs() <= tolerance,
        "{context}: got {actual}, brute force says {expected}"
    );
}

fuzz_target!(|tuple: (LineString<f64>, LineString<f64>)| {
    let (a, mut b) = tuple;

    if a.0.len() < 2 || b.0.len() < 2 || !in_bounds(&a) || !in_bounds(&b) {
        return;
    }

    // Translate `b` so that the bounding boxes are separated along x, which selects the
    // fast path. The y extents remain free to overlap, which is what exercises the
    // pruning
    let a_max_x = a.coords().fold(f64::NEG_INFINITY, |acc, c| acc.max(c.x));
    let b_min_x = b.coords().fold(f64::INFINITY, |acc, c| acc.min(c.x));
    let shift = (a_max_x + 1.0) - b_min_x;
    for coord in &mut b.0 {
        coord.x += shift;
    }

    let expected = brute_force_line_string_distance(&a, &b);
    assert_close(Euclidean.distance(&a, &b), expected, "line string a-b");
    assert_close(Euclidean.distance(&b, &a), expected, "line string b-a");

    // Closed rings take the same code path under a different adjacency rule. The boxes
    // are x-separated, so neither ring can contain the other
    if a.0.len() >= 3 && b.0.len() >= 3 {
        let (mut ring_a, mut ring_b) = (a, b);
        ring_a.close();
        ring_b.close();
        let expected = brute_force_line_string_distance(&ring_a, &ring_b);
        let poly_a = Polygon::new(ring_a, vec![]);
        let poly_b = Polygon::new(ring_b, vec![]);
        assert_close(Euclidean.distance(&poly_a, &poly_b), expected, "polygon");
    }
});
