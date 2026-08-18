//! Converts the node sections at a polygon node where a shell and one or
//! more holes touch, or two or more holes touch.
//!
//! Port of JTS `PolygonNodeConverter`.
//!
//! This converts the node topological structure from the OGC
//! "touching-rings" (minimal-ring) model to the equivalent "self-touch"
//! (maximal-ring) model. In the self-touch model the converted section
//! corners enclose areas which all lie inside the polygon (they do not
//! enclose hole edges). This allows `RelateNode` to use simple
//! area-additive semantics for adding edges and propagating edge locations.
//!
//! The input node sections must have canonical orientation (CW shells and
//! CCW holes), and the arrangement of shells and holes must be
//! topologically valid: the node sections must not cross or be collinear.

use crate::dimensions::Dimensions;
use crate::{Coord, GeoFloat};

use super::node_section::NodeSection;

/// Converts a list of sections of valid polygon rings to have
/// "self-touching" structure. There are the same number of output sections
/// as input ones.
pub(crate) fn convert<F: GeoFloat>(mut poly_sections: Vec<NodeSection<F>>) -> Vec<NodeSection<F>> {
    poly_sections.sort_by(|a, b| a.compare_by_edge_angle(b));

    let sections = extract_unique(poly_sections);
    if sections.len() == 1 {
        return sections;
    }

    let Some(shell_index) = find_shell(&sections) else {
        return convert_holes(&sections);
    };
    // At least one shell is present. Handle multiple ones if present.
    let mut converted = Vec::with_capacity(sections.len());
    let mut next_shell_index = shell_index;
    loop {
        next_shell_index = convert_shell_and_holes(&sections, next_shell_index, &mut converted);
        if next_shell_index == shell_index {
            break;
        }
    }
    converted
}

/// Converts the corners around one shell section, pairing its in-vertex
/// with the out-vertices of the intervening hole sections. Returns the
/// index of the next shell section.
fn convert_shell_and_holes<F: GeoFloat>(
    sections: &[NodeSection<F>],
    shell_index: usize,
    converted: &mut Vec<NodeSection<F>>,
) -> usize {
    let shell_section = &sections[shell_index];
    let mut in_vertex = expect_vertex(shell_section, 0);
    let mut i = next(sections.len(), shell_index);
    while !sections[i].is_shell() {
        let hole_section = &sections[i];
        let out_vertex = expect_vertex(hole_section, 1);
        converted.push(create_section(shell_section, in_vertex, out_vertex));

        in_vertex = expect_vertex(hole_section, 0);
        i = next(sections.len(), i);
    }
    // Create the final section for the corner from the last hole to the
    // shell.
    let out_vertex = expect_vertex(shell_section, 1);
    converted.push(create_section(shell_section, in_vertex, out_vertex));
    i
}

fn convert_holes<F: GeoFloat>(sections: &[NodeSection<F>]) -> Vec<NodeSection<F>> {
    let mut converted = Vec::with_capacity(sections.len());
    let copy_section = &sections[0];
    for i in 0..sections.len() {
        let i_next = next(sections.len(), i);
        let in_vertex = expect_vertex(&sections[i], 0);
        let out_vertex = expect_vertex(&sections[i_next], 1);
        converted.push(create_section(copy_section, in_vertex, out_vertex));
    }
    converted
}

/// A converted section: an area shell section with the given corner
/// vertices, carrying the source section's identity.
fn create_section<F: GeoFloat>(ns: &NodeSection<F>, v0: Coord<F>, v1: Coord<F>) -> NodeSection<F> {
    NodeSection::new(
        ns.input(),
        Dimensions::TwoDimensional,
        ns.id(),
        0,
        ns.polygonal_id(),
        ns.is_node_at_vertex(),
        Some(v0),
        ns.node_pt(),
        Some(v1),
    )
}

/// Drops consecutive duplicate sections from an angle-sorted list.
fn extract_unique<F: GeoFloat>(sections: Vec<NodeSection<F>>) -> Vec<NodeSection<F>> {
    let mut unique_sections: Vec<NodeSection<F>> = Vec::with_capacity(sections.len());
    for ns in sections {
        match unique_sections.last() {
            Some(last_unique) if last_unique == &ns => continue,
            _ => unique_sections.push(ns),
        }
    }
    unique_sections
}

fn next(len: usize, i: usize) -> usize {
    let next = i + 1;
    if next >= len { 0 } else { next }
}

fn find_shell<F: GeoFloat>(poly_sections: &[NodeSection<F>]) -> Option<usize> {
    poly_sections.iter().position(|ns| ns.is_shell())
}

fn expect_vertex<F: GeoFloat>(ns: &NodeSection<F>, i: usize) -> Coord<F> {
    ns.vertex(i)
        .expect("polygon node sections have both incident edges")
}

#[cfg(test)]
mod tests {
    // Tests ported from JTS PolygonNodeConverterTest.java (master, ab57bff).
    use super::super::topology_predicate::InputIndex;
    use super::*;

    fn section(
        ring_id: i32,
        v0x: f64,
        v0y: f64,
        nx: f64,
        ny: f64,
        v1x: f64,
        v1y: f64,
    ) -> NodeSection<f64> {
        NodeSection::new(
            InputIndex::A,
            Dimensions::TwoDimensional,
            1,
            ring_id,
            None,
            false,
            Some(Coord { x: v0x, y: v0y }),
            Coord { x: nx, y: ny },
            Some(Coord { x: v1x, y: v1y }),
        )
    }

    fn section_shell(v0x: f64, v0y: f64, nx: f64, ny: f64, v1x: f64, v1y: f64) -> NodeSection<f64> {
        section(0, v0x, v0y, nx, ny, v1x, v1y)
    }

    fn section_hole(v0x: f64, v0y: f64, nx: f64, ny: f64, v1x: f64, v1y: f64) -> NodeSection<f64> {
        section(1, v0x, v0y, nx, ny, v1x, v1y)
    }

    fn check_conversion(input: Vec<NodeSection<f64>>, expected: Vec<NodeSection<f64>>) {
        let mut actual = convert(input);
        let mut expected = expected;
        actual.sort_by(|a, b| a.compare_by_edge_angle(b));
        expected.sort_by(|a, b| a.compare_by_edge_angle(b));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_shells() {
        check_conversion(
            vec![
                section_shell(1., 1., 5., 5., 9., 9.),
                section_shell(8., 9., 5., 5., 6., 9.),
                section_shell(4., 9., 5., 5., 2., 9.),
            ],
            vec![
                section_shell(1., 1., 5., 5., 9., 9.),
                section_shell(8., 9., 5., 5., 6., 9.),
                section_shell(4., 9., 5., 5., 2., 9.),
            ],
        );
    }

    #[test]
    fn test_shell_and_hole() {
        check_conversion(
            vec![
                section_shell(1., 1., 5., 5., 9., 9.),
                section_hole(6., 0., 5., 5., 4., 0.),
            ],
            vec![
                section_shell(1., 1., 5., 5., 4., 0.),
                section_shell(6., 0., 5., 5., 9., 9.),
            ],
        );
    }

    #[test]
    fn test_shells_and_holes() {
        check_conversion(
            vec![
                section_shell(1., 1., 5., 5., 9., 9.),
                section_hole(6., 0., 5., 5., 4., 0.),
                section_shell(8., 8., 5., 5., 1., 8.),
                section_hole(4., 8., 5., 5., 6., 8.),
            ],
            vec![
                section_shell(1., 1., 5., 5., 4., 0.),
                section_shell(6., 0., 5., 5., 9., 9.),
                section_shell(4., 8., 5., 5., 1., 8.),
                section_shell(8., 8., 5., 5., 6., 8.),
            ],
        );
    }

    #[test]
    fn test_shell_and_2_holes() {
        check_conversion(
            vec![
                section_shell(1., 1., 5., 5., 9., 9.),
                section_hole(7., 0., 5., 5., 6., 0.),
                section_hole(4., 0., 5., 5., 3., 0.),
            ],
            vec![
                section_shell(1., 1., 5., 5., 3., 0.),
                section_shell(4., 0., 5., 5., 6., 0.),
                section_shell(7., 0., 5., 5., 9., 9.),
            ],
        );
    }

    #[test]
    fn test_holes() {
        check_conversion(
            vec![
                section_hole(7., 0., 5., 5., 6., 0.),
                section_hole(4., 0., 5., 5., 3., 0.),
            ],
            vec![
                section_shell(4., 0., 5., 5., 6., 0.),
                section_shell(7., 0., 5., 5., 3., 0.),
            ],
        );
    }
}
