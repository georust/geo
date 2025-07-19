#![allow(dead_code)]

use geo::algorithm::sweep::Intersections;
use geo::{line_intersection::line_intersection, Line};

use rstar::{primitives::GeomWithData, RTree};

pub fn count_bo(lines: Vec<Line<f64>>) -> usize {
    Intersections::from_iter(lines).count()
}

pub fn count_brute(lines: &[Line<f64>]) -> usize {
    let mut count = 0;
    let n = lines.len();
    for i in 0..n {
        let l1 = &lines[i];
        for l2 in lines.iter().take(n).skip(i + 1) {
            if line_intersection(*l1, *l2).is_some() {
                count += 1;
            }
        }
    }
    count
}

pub fn count_rtree(lines: &[Line<f64>]) -> usize {
    let lines: Vec<_> = lines
        .iter()
        .enumerate()
        .map(|(i, l)| GeomWithData::new(*l, i))
        .collect();

    let tree = RTree::bulk_load(lines);
    tree.intersection_candidates_with_other_tree(&tree)
        .filter_map(|(l1, l2)| {
            if l1.data >= l2.data {
                None
            } else {
                line_intersection(*l1.geom(), *l2.geom())
            }
        })
        .count()
}
