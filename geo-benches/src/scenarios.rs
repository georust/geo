//! Realistic profiling workloads for identifying optimisation candidates.
//!
//! Each [`Scenario`] pairs one-off setup (fixture parsing, input construction)
//! with a closure that performs a single iteration of the workload, so that
//! setup cost is excluded from what a profiler samples. The `profiling` binary
//! (`src/bin/profiling.rs`) drives these under a sampling profiler such as
//! samply or Instruments; a future gungraun bench target can wrap the same
//! functions for instruction-level measurement on Linux.
//!
//! Scenarios are chosen algorithm-first: computationally heavy operations on
//! large real-world fixtures, including algorithms with no criterion coverage
//! (`Buffer`, `MakeValid`, `KNearestConcaveHull`, `HausdorffDistance`).
//! Inputs are deterministic – fixtures are static and random inputs use a
//! seeded RNG – so profiles are comparable across runs.

use std::hint::black_box;

use geo::algorithm::sweep::Intersections;
use geo::algorithm::{
    BooleanOps, BoundingRect, Buffer, Centroid, ConcaveHull, Contains, Distance, Euclidean,
    HausdorffDistance, KNearestConcaveHull, MakeValid, MinimumRotatedRect, Relate,
    SimplifyVwPreserve, Translate, TriangulateDelaunay, TriangulateEarcut, Validation, unary_union,
};
use geo::geometry::{Coord, Line, LineString, MultiPoint, MultiPolygon, Point, Polygon, Rect};
use geo::line_measures::FrechetDistance;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::utils::random::uniform_line;

/// A named profiling workload: `prepare` performs setup and returns a closure
/// executing one iteration.
pub struct Scenario {
    pub name: &'static str,
    pub description: &'static str,
    pub prepare: fn() -> Box<dyn FnMut()>,
}

/// All scenarios, in rough priority order for a profiling sweep.
pub fn all() -> Vec<Scenario> {
    vec![
        Scenario {
            name: "boolean-ops-nl-zones",
            description: "union + intersection of nl_zones against a translated copy",
            prepare: boolean_ops_nl_zones,
        },
        Scenario {
            name: "boolean-ops-asia",
            description: "unary union of the asia.geojson polygons",
            prepare: boolean_ops_asia,
        },
        Scenario {
            name: "relate-jts",
            description: "run the entire JTS test suite via jts-test-runner",
            prepare: relate_jts,
        },
        Scenario {
            name: "relate-nl-plots",
            description: "relate each adjacent polygon pair in nl_plots",
            prepare: relate_nl_plots,
        },
        Scenario {
            name: "buffer-nl-zones",
            description: "buffer the nl_zones MultiPolygon at two distances",
            prepare: buffer_nl_zones,
        },
        Scenario {
            name: "buffer-norway",
            description: "buffer a subsampled norway_main LineString",
            prepare: buffer_norway,
        },
        Scenario {
            name: "make-valid-checkerboard",
            description: "MakeValid on a level-5 checkerboard polygon",
            prepare: make_valid_checkerboard,
        },
        Scenario {
            name: "make-valid-norway-zigzag",
            description: "MakeValid on a heavily self-intersecting reordering of norway_main",
            prepare: make_valid_norway_zigzag,
        },
        Scenario {
            name: "triangulate-earcut-nl-zones",
            description: "earcut triangulation of every nl_zones polygon",
            prepare: triangulate_earcut_nl_zones,
        },
        Scenario {
            name: "triangulate-cdt-nl-zones",
            description: "constrained Delaunay triangulation of every nl_zones polygon",
            prepare: triangulate_cdt_nl_zones,
        },
        Scenario {
            name: "concave-hull-norway",
            description: "concave hull of the norway_main LineString",
            prepare: concave_hull_norway,
        },
        Scenario {
            name: "knn-hull-norway",
            description: "k-nearest concave hull (k=16) of subsampled norway_main points",
            prepare: knn_hull_norway,
        },
        Scenario {
            name: "sweep-crossings-1k",
            description: "sweep-line intersections over 1024 random lines",
            prepare: sweep_crossings_1k,
        },
        Scenario {
            name: "sweep-crossings-4k",
            description: "sweep-line intersections over 4096 random lines",
            prepare: sweep_crossings_4k,
        },
        Scenario {
            name: "sweep-crossings-16k",
            description: "sweep-line intersections over 16384 random lines",
            prepare: sweep_crossings_16k,
        },
        Scenario {
            name: "contains-grid-nl-zones",
            description: "point-in-polygon over a 96x96 grid against nl_zones (unindexed)",
            prepare: contains_grid_nl_zones,
        },
        Scenario {
            name: "euclidean-distance-norway-louisiana",
            description: "minimum distance between the norway_main and louisiana polygons",
            prepare: euclidean_distance_norway_louisiana,
        },
        Scenario {
            name: "hausdorff-norway-louisiana",
            description: "Hausdorff distance between norway_main and louisiana",
            prepare: hausdorff_norway_louisiana,
        },
        Scenario {
            name: "frechet-vw",
            description: "Frechet distance between vw_orig and vw_simplified",
            prepare: frechet_vw,
        },
        Scenario {
            name: "validation-nl-zones",
            description: "validate the nl_zones MultiPolygon",
            prepare: validation_nl_zones,
        },
        Scenario {
            name: "simplify-vw-norway",
            description: "topology-preserving Visvalingam-Whyatt simplification of norway_main",
            prepare: simplify_vw_norway,
        },
        Scenario {
            name: "centroid-nl-zones",
            description: "centroid of the nl_zones MultiPolygon",
            prepare: centroid_nl_zones,
        },
        Scenario {
            name: "bounding-rect-nl-zones",
            description: "bounding rectangle of the nl_zones MultiPolygon",
            prepare: bounding_rect_nl_zones,
        },
        Scenario {
            name: "minimum-rotated-rect-nl-zones",
            description: "minimum rotated rectangle of the nl_zones MultiPolygon",
            prepare: minimum_rotated_rect_nl_zones,
        },
    ]
}

/// Reduce a LineString to roughly `target` evenly strided coordinates.
fn subsample(ls: &LineString<f64>, target: usize) -> Vec<Coord<f64>> {
    let stride = (ls.0.len() / target).max(1);
    ls.0.iter().step_by(stride).copied().collect()
}

fn boolean_ops_nl_zones() -> Box<dyn FnMut()> {
    let a: MultiPolygon<f64> = geo_test_fixtures::nl_zones();
    let b = a.translate(0.05, 0.05);
    Box::new(move || {
        black_box(a.union(&b));
        black_box(a.intersection(&b));
    })
}

fn boolean_ops_asia() -> Box<dyn FnMut()> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fixtures/rust-geo-booleanop-fixtures/benchmarks/asia.geojson"
    );
    let gj: geojson::GeoJson = std::fs::read_to_string(path)
        .expect("read asia.geojson")
        .parse()
        .expect("parse asia.geojson");
    let mut polys: Vec<Polygon<f64>> = Vec::new();
    if let geojson::GeoJson::FeatureCollection(fc) = gj {
        for feature in fc.features {
            if let Some(geometry) = feature.geometry {
                match geo_types::Geometry::<f64>::try_from(geometry) {
                    Ok(geo_types::Geometry::Polygon(p)) => polys.push(p),
                    Ok(geo_types::Geometry::MultiPolygon(mp)) => polys.extend(mp),
                    _ => {}
                }
            }
        }
    }
    assert!(!polys.is_empty(), "no polygons found in asia.geojson");
    Box::new(move || {
        black_box(unary_union(&polys));
    })
}

fn relate_jts() -> Box<dyn FnMut()> {
    let mut runner = jts_test_runner::TestRunner::new();
    runner.prepare_cases().expect("prepare JTS cases");
    Box::new(move || {
        let mut r = runner.clone();
        r.run().expect("run JTS suite");
        black_box(r.successes().len());
    })
}

fn relate_nl_plots() -> Box<dyn FnMut()> {
    let plots = geo_test_fixtures::nl_plots_wgs84::<f64>().0;
    Box::new(move || {
        for pair in plots.windows(2) {
            black_box(pair[0].relate(&pair[1]));
        }
    })
}

fn buffer_nl_zones() -> Box<dyn FnMut()> {
    let zones: MultiPolygon<f64> = geo_test_fixtures::nl_zones();
    Box::new(move || {
        black_box(zones.buffer(0.001));
        black_box(zones.buffer(0.01));
    })
}

fn buffer_norway() -> Box<dyn FnMut()> {
    let norway: LineString<f64> = geo_test_fixtures::norway_main();
    let line = LineString::from(subsample(&norway, 4096));
    Box::new(move || {
        black_box(line.buffer(0.05));
    })
}

fn make_valid_checkerboard() -> Box<dyn FnMut()> {
    let poly = geo_test_fixtures::checkerboard::create_checkerboard_polygon(5);
    Box::new(move || {
        black_box(poly.make_valid().ok());
    })
}

fn make_valid_norway_zigzag() -> Box<dyn FnMut()> {
    let norway: LineString<f64> = geo_test_fixtures::norway_main();
    let pts = subsample(&norway, 3000);
    // Interleave even-indexed points with reversed odd-indexed points to
    // produce a ring with a large number of self-intersections
    let mut ring: Vec<Coord<f64>> = pts.iter().copied().step_by(2).collect();
    ring.extend(pts.iter().copied().skip(1).step_by(2).rev());
    let poly = Polygon::new(LineString::from(ring), vec![]);
    Box::new(move || {
        black_box(poly.make_valid().ok());
    })
}

fn triangulate_earcut_nl_zones() -> Box<dyn FnMut()> {
    let zones: MultiPolygon<f64> = geo_test_fixtures::nl_zones();
    Box::new(move || {
        for poly in &zones {
            black_box(poly.earcut_triangles());
        }
    })
}

fn triangulate_cdt_nl_zones() -> Box<dyn FnMut()> {
    let zones: MultiPolygon<f64> = geo_test_fixtures::nl_zones();
    Box::new(move || {
        for poly in &zones {
            black_box(poly.constrained_triangulation(Default::default()).ok());
        }
    })
}

fn concave_hull_norway() -> Box<dyn FnMut()> {
    let norway: LineString<f64> = geo_test_fixtures::norway_main();
    Box::new(move || {
        black_box(norway.concave_hull());
    })
}

fn knn_hull_norway() -> Box<dyn FnMut()> {
    let norway: LineString<f64> = geo_test_fixtures::norway_main();
    let points: MultiPoint<f64> = MultiPoint(
        subsample(&norway, 3000)
            .into_iter()
            .map(Point::from)
            .collect(),
    );
    Box::new(move || {
        black_box(points.k_nearest_concave_hull(16));
    })
}

fn sweep_lines(count: usize) -> Vec<Line<f64>> {
    let mut rng = StdRng::seed_from_u64(42);
    let bounds = Rect::new(Coord { x: -100., y: -100. }, Coord { x: 100., y: 100. });
    (0..count).map(|_| uniform_line(&mut rng, bounds)).collect()
}

fn sweep_crossings(count: usize) -> Box<dyn FnMut()> {
    let lines = sweep_lines(count);
    Box::new(move || {
        black_box(Intersections::from_iter(&lines).count());
    })
}

fn sweep_crossings_1k() -> Box<dyn FnMut()> {
    sweep_crossings(1024)
}

fn sweep_crossings_4k() -> Box<dyn FnMut()> {
    sweep_crossings(4096)
}

fn sweep_crossings_16k() -> Box<dyn FnMut()> {
    sweep_crossings(16384)
}

fn contains_grid_nl_zones() -> Box<dyn FnMut()> {
    let zones: MultiPolygon<f64> = geo_test_fixtures::nl_zones();
    let rect = zones.bounding_rect().expect("nl_zones bounding rect");
    let steps = 96;
    let dx = rect.width() / steps as f64;
    let dy = rect.height() / steps as f64;
    let points: Vec<Point<f64>> = (0..steps)
        .flat_map(|i| {
            (0..steps).map(move |j| {
                Point::new(
                    rect.min().x + dx * (i as f64 + 0.5),
                    rect.min().y + dy * (j as f64 + 0.5),
                )
            })
        })
        .collect();
    Box::new(move || {
        let mut inside = 0usize;
        for point in &points {
            if zones.contains(point) {
                inside += 1;
            }
        }
        black_box(inside);
    })
}

fn euclidean_distance_norway_louisiana() -> Box<dyn FnMut()> {
    let norway = Polygon::new(geo_test_fixtures::norway_main::<f64>(), vec![]);
    let louisiana = Polygon::new(geo_test_fixtures::louisiana::<f64>(), vec![]);
    Box::new(move || {
        black_box(Euclidean.distance(&norway, &louisiana));
    })
}

fn hausdorff_norway_louisiana() -> Box<dyn FnMut()> {
    let norway: LineString<f64> = geo_test_fixtures::norway_main();
    let louisiana: LineString<f64> = geo_test_fixtures::louisiana();
    Box::new(move || {
        black_box(norway.hausdorff_distance(&louisiana));
    })
}

fn frechet_vw() -> Box<dyn FnMut()> {
    let a: LineString<f64> = geo_test_fixtures::vw_orig();
    let b: LineString<f64> = geo_test_fixtures::vw_simplified();
    Box::new(move || {
        black_box(Euclidean.frechet_distance(&a, &b));
    })
}

fn validation_nl_zones() -> Box<dyn FnMut()> {
    let zones: MultiPolygon<f64> = geo_test_fixtures::nl_zones();
    Box::new(move || {
        black_box(zones.is_valid());
    })
}

fn simplify_vw_norway() -> Box<dyn FnMut()> {
    let norway: LineString<f64> = geo_test_fixtures::norway_main();
    Box::new(move || {
        black_box(norway.simplify_vw_preserve(0.0005));
    })
}

fn centroid_nl_zones() -> Box<dyn FnMut()> {
    let zones: MultiPolygon<f64> = geo_test_fixtures::nl_zones();
    Box::new(move || {
        black_box(zones.centroid());
    })
}

fn bounding_rect_nl_zones() -> Box<dyn FnMut()> {
    let zones: MultiPolygon<f64> = geo_test_fixtures::nl_zones();
    Box::new(move || {
        black_box(zones.bounding_rect());
    })
}

fn minimum_rotated_rect_nl_zones() -> Box<dyn FnMut()> {
    let zones: MultiPolygon<f64> = geo_test_fixtures::nl_zones();
    Box::new(move || {
        black_box(zones.minimum_rotated_rect());
    })
}
