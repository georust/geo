//! Determines the boundary points of a linear geometry.
//!
//! Port of JTS `LinearBoundary`, specialised to the Mod-2 (OGC SFS)
//! boundary node rule: an endpoint is on the boundary iff an odd number of
//! line ends meet there. JTS supports pluggable rules; the rule reduces to
//! the single function [`is_in_boundary_mod2`], so alternatives can be
//! added later without structural change.

use std::collections::BTreeMap;

use crate::relate::geomgraph::node_map::NodeKey;
use crate::{Coord, GeoFloat, LineString};

/// The Mod-2 (OGC SFS) boundary node rule.
fn is_in_boundary_mod2(degree: usize) -> bool {
    degree % 2 == 1
}

#[derive(Clone)]
pub(crate) struct LinearBoundary<F: GeoFloat> {
    vertex_degree: BTreeMap<NodeKey<F>, usize>,
    has_boundary: bool,
}

impl<F: GeoFloat> LinearBoundary<F> {
    /// Computes the endpoint degrees of a set of lines. The input must be
    /// the linear components of a one-dimensional geometry.
    pub fn new<'a>(lines: impl IntoIterator<Item = &'a LineString<F>>) -> Self
    where
        F: 'a,
    {
        let vertex_degree = compute_boundary_points(lines);
        let has_boundary = vertex_degree
            .values()
            .any(|&degree| is_in_boundary_mod2(degree));
        Self {
            vertex_degree,
            has_boundary,
        }
    }

    /// Whether any endpoint is a boundary point. A closed line, for
    /// example, has no boundary under the Mod-2 rule.
    pub fn has_boundary(&self) -> bool {
        self.has_boundary
    }

    pub fn is_boundary(&self, pt: Coord<F>) -> bool {
        match self.vertex_degree.get(&NodeKey(pt)) {
            Some(&degree) => is_in_boundary_mod2(degree),
            None => false,
        }
    }
}

fn compute_boundary_points<'a, F: GeoFloat + 'a>(
    lines: impl IntoIterator<Item = &'a LineString<F>>,
) -> BTreeMap<NodeKey<F>, usize> {
    let mut vertex_degree = BTreeMap::new();
    for line in lines {
        if line.0.is_empty() {
            continue;
        }
        add_endpoint(line.0[0], &mut vertex_degree);
        add_endpoint(line.0[line.0.len() - 1], &mut vertex_degree);
    }
    vertex_degree
}

fn add_endpoint<F: GeoFloat>(p: Coord<F>, degree: &mut BTreeMap<NodeKey<F>, usize>) {
    *degree.entry(NodeKey(p)).or_insert(0) += 1;
}

#[cfg(test)]
mod tests {
    // Tests ported from JTS LinearBoundaryTest.java (master, ab57bff).
    // testLines3Monvalent is not ported: it exercises the
    // MONOVALENT_ENDPOINT_BOUNDARY_RULE, and this port supports the Mod-2
    // rule only.
    use super::*;
    use crate::wkt;
    use crate::{MultiLineString, MultiPoint};

    fn check_linear_boundary(lines: Vec<LineString<f64>>, expected_boundary: MultiPoint<f64>) {
        let lb = LinearBoundary::new(lines.iter());
        let boundary_coords: Vec<Coord<f64>> = expected_boundary.0.iter().map(|p| p.0).collect();
        assert_eq!(
            lb.has_boundary(),
            !boundary_coords.is_empty(),
            "has_boundary"
        );

        for &p in &boundary_coords {
            assert!(lb.is_boundary(p), "expected boundary point {p:?}");
        }
        for line in &lines {
            for &p in &line.0 {
                if !boundary_coords.contains(&p) {
                    assert!(!lb.is_boundary(p), "unexpected boundary point {p:?}");
                }
            }
        }
    }

    #[test]
    fn test_line_mod2() {
        check_linear_boundary(
            vec![wkt!(LINESTRING (0. 0., 9. 9.))],
            wkt!(MULTIPOINT(0. 0., 9. 9.)),
        );
    }

    #[test]
    fn test_lines_2_mod2() {
        let mls: MultiLineString<f64> = wkt!(MULTILINESTRING ((0. 0., 9. 9.), (9. 9., 5. 1.)));
        check_linear_boundary(mls.0, wkt!(MULTIPOINT(0. 0., 5. 1.)));
    }

    #[test]
    fn test_lines_3_mod2() {
        let mls: MultiLineString<f64> =
            wkt!(MULTILINESTRING ((0. 0., 9. 9.), (9. 9., 5. 1.), (9. 9., 1. 5.)));
        check_linear_boundary(mls.0, wkt!(MULTIPOINT(0. 0., 5. 1., 1. 5., 9. 9.)));
    }
}
