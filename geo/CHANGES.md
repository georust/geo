# Changes

## Unreleased

- Add simply connected interior validation for polygons. Polygons with holes that touch at vertices in ways that disconnect the interior (e.g., two holes sharing 2+ vertices, or cycles of holes each sharing a vertex) are now detected as invalid via `Validation::is_valid()`. This aligns with OGC Simple Features (ISO 19125-1, section 6.1.11.1) and matches PostGIS behavior.
- Polygon validation now uses `PreparedGeometry` to cache R-tree structures for interior/exterior containment checks, improving validation speed for polygons with many holes.
  - <https://github.com/georust/geo/pull/1501>
- Update `i_overlay` to 4.4 and enable OGC-compliant polygon extraction for all boolean operations, fixing cases where holes sharing vertices produced invalid geometry.
  - <https://github.com/georust/geo/pull/1500>
- Fix `CoordinatePosition` for `LineString` to handle dimensionally collapsed input  e.g. `LINESTRING(0 0)` is treated like `POINT(0 0)`.
  - <https://github.com/georust/geo/pull/1483>
- Fix `CoordinatePosition` for `Triangle` to correctly return `CoordPos::OnBoundary` for coordinate within vertical segment.
- Split `TriangulateDelaunay` trait to support Point collections in addition to existing geometries
  - <https://github.com/georust/geo/pull/1486>
- Use Coord as DelaunayTriangulation vertex type
  - <https://github.com/georust/geo/pull/1490>
- Add Voronoi diagram generation functionality
  - <https://github.com/georust/geo/pull/1487>
- Fix Euclidean distance fast path for open `LineString`s to consider the last vertex (avoids incorrect `LineString`-to-`LineString` distances for separable geometries).
  - <https://github.com/georust/geo/pull/1499>

## 0.32.0 - 2025-12-05

- Move `PreparedGeometry` into a new `indexed` module intended to provide index-backed geometries. `relate::PreparedGeometry` has been deprecated.
- Use an interval tree for faster (Multi)Point in MultiPolygon checks
- LOF algorithm efficiency improvements due to caching kth distance
- Add DBSCAN clustering algorithm implementation
- Add `distance_within` method with default impl for any geometry that implements `Distance`, with similar semantics to the PostGIS [ST_DWithin](https://postgis.net/docs/ST_DWithin.html) function
- Add fast minimum 1D and 2D Euclidean distance algorithm for linearly separable geometries (#1424)
- Add `ContainsProperly` trait to relate and as a standalone operation
  - `ContainsProperly` is faster for `Polygon` and `MultiPolygon` when inputs are smaller than about 650 vertices, otherwise use `Relate.is_contains_properly`
  - <https://github.com/georust/geo/pull/1457>
- Add k-means clustering algorithm
- POSSIBLY BREAKING: `minimum_rotated_rect` is about 1.3-2x faster, but might return slightly different results.
  - <https://github.com/georust/geo/pull/1446>
- Renamed features `use-proj`, `use-serde` to simply `proj` and `serde` (removing the `use-` prefix)
  and deprecated the old spelling.
  - <https://github.com/georust/geo/pull/1447>
- Update `ConcaveHull` algorithm with implementation of [mapbox/concaveman](https://github.com/mapbox/concaveman).
  - BREAKING: The `concave_hull` method now has no `concavity` parameter.  
  - Add `concave_hull_with_options` method which requires `ConcaveHullOptions` as a parameter with `concavity` and `length_threshold` options.
  - <https://github.com/georust/geo/pull/1442>
- Add `Covers` trait to relate and as a standalone operation
  - custom implementations for checking Geometries covered by `Rect`, `Triangle`, `Line`, `Point`, `Coord`
  - custom implementations for checking Geometries covering `Point` and `MultiPoint`

## 0.31.0 - 2025-09-01

- Added: Geometry buffering to "grow" or "shrink" a geometry by creating a buffer whose boundary is the specified offset from the input.
  - <https://github.com/georust/geo/pull/1365>
- BREAKING: `BoolOpsNum` must now implement GeoFloat, not just GeoNum. In practice, this shouldn't break for any concrete types (like f32, f64).
  - <https://github.com/georust/geo/pull/1365>
- BREAKING: The `Simplify`, `SimplifyVw`, and `SimplifyVwIdx` traits no longer require a borrowed `epsilon` parameter as these are `Copy` types
- Performance: Reduce memory consumption of FrechetDistance calculation.
  - <https://github.com/georust/geo/pull/1357>
- Bump geo MSRV to 1.85
- Update i_overlay (dependency of BooleanOps and Buffer).
  This is *mostly* an internal change. However, if you are using i_overlay directly in your project,
  you'll notice that the `FillRule`, `LineCap`, and `LineJoin` options, which are re-exported from i_overlay,
  are now compatible with 4.0
  <https://github.com/georust/geo/pull/1405>
- Simplify test rustc and libproj version specification in CI
- Performance: Avoid running through entire iterator to reach last element in `outlier_detection` when calculating LRD and LOF
- Add `Bearing` and `Destination` trait implementations for `Euclidean`
- Add `FillRule`-configurable boolean operations to `BooleanOps` trait
  - <https://github.com/georust/geo/pull/1382>
- Fix panic in `algorithm::simplify::compute_rdp` with one point
- Fix Clippy warning (surfaced in Rust 1.89) related to lifetime elision
- Silence Clippy warnings related to `old_sweep` module
- Fix false positive convexity check for "star polygon" LineStrings
  - BREAKING: previously, an empty LineString was considered non-convex. This has changed: empty LineStrings are now considered convex, in line with tools such as PostGIS
- Update to proj 0.31.0 (libproj 9.6.2)
- Update `Intersections` with new implementation of the Bentley-Ottmann sweep-line algorithm to efficiently find sparse intersections between groups of lines.
  - no longer `panic`'s when given pathological input
  - BREAKING: `Intersections` now computes intersections lazily
  - BREAKING: The `Crosses` trait used by `Intersections` now returns a `Line`, not a `LineOrPoint`.
  - BREAKING: The `Crosses` trait no longer needs to implement Clone
    - <https://github.com/georust/geo/pull/1358>
    - <https://github.com/georust/geo/pull/1387>
    - <https://github.com/georust/geo/pull/1359>
- Fix `is_convex` to correctly handle duplicate points
- Fix`graham_hull` to correctly handle duplicate points when `on_hull` is set to true
  - `graham_hull` now always returns a boundary with no duplicated points
  - <https://github.com/georust/geo/issues/1383>
- BREAKING: Break up blanket implementation of `Intersects<LineString>` into specific traits
  - faster implementations for `Rect`, `Triangle`, `MultiPolygon`, `Polygon` intersects `LineString`
  - <https://github.com/georust/geo/pull/1379>
- Added `Contains` implementation for all remaining geometries.

## 0.30.0 - 2025-03-24

- Bump `geo` MSRV to 1.81
- BREAKING: Update proj dependency to 0.29.0
- BREAKING: `FrechetDistance` is now defined on the metric space, rather than a method on a Linestring.
  - <https://github.com/georust/geo/pull/1274>
  ```rust
  // before (implicitly Euclidean)
  line_string_1.frechet_distance(&line_string_2);

  // after
  Euclidean.frechet_distance(&line_string_1, &line_string_2);
  Haversine.frechet_distance(&line_string_1, &line_string_2);
  ```
- BREAKING: `Densify` and `Length` are now defined on the metric space, rather than a generic method on the geometry.
  - <https://github.com/georust/geo/pull/1298>
  ```rust
  // before
  line_string.length::<Euclidean>()
  line_string.densify::<Euclidean>()
  line_string.densify::<Haversine>()

  // after
  Euclidean.length(&line_string)
  Euclidean.densify(&line_string)
  Haversine.densify(&line_string)
  ```
- Add configurable `HaversineMeasure` for doing calculations on a custom sphere. Use the `Haversine` instance for the default Earth radius.
  - <https://github.com/georust/geo/pull/1298>
  ```rust
  // before
  Haversine::distance(point1, point2)

  // after
  Haversine.distance(point1, point2)

  // For custom earth (or non-earth!) radius
  HaversineMeasure::new(3_389_500.0).distance(point1, point2)
  ```
- Add configurable `GeodesicMeasure` for doing calculations on a custom geoid. Use the `Geodesic` instance for the default Earth geoid.
  - <https://github.com/georust/geo/pull/1311>
  ```rust
  // before
  Geodesic::distance(point1, point2)

  // after
  Geodesic.distance(point1, point2)

  // For custom Earth (or non-earth!) geoids:
  let nad83_flattening = 1. / 298.257222101;
  GeodesicMeasure::new(6_378_137, nad83_flattening).distance(point1, point2)
  ```
- Add `InterpolateLine` to interpolate a point along a line using Euclidean, Haversine, Geodesic, Rhumb metric spaces.
- Deprecate `LineInterpolatePoint` which was implicitly Euclidean only.
- Fix bug in `SegmentizeHaversine` which caused segments to be unequal lengths
  - <https://github.com/georust/geo/pull/1321>
- Rename `triangulate_spade` and `TriangulateSpade` to `triangulate_delaunay` and `TriangulateDelaunay`
- Docs: Fix page location of citation for mean earth radius used in Haversine calculations
  - <https://github.com/georust/geo/pull/1297>
- Docs: Add top-level doc link for `InteriorPoint`
- Add Unary Union algorithm for fast union ops on adjacent / overlapping geometries
  - <https://github.com/georust/geo/pull/1246>
- Loosen bounds on `RemoveRepeatedPoints` trait (`num_traits::FromPrimitive` isn't required)
  - <https://github.com/georust/geo/pull/1278>
- Fix a math error in some rhumb line calculations
  - <https://github.com/georust/geo/pull/1330>
- Added: `Validation` trait to check validity of `Geometry`.
  - https://github.com/georust/geo/pull/1279
  ```rust
  // use in control flow
  if polygon.is_valid() { foo() }

  // raise an error if invalid
  polygon.check_validation()?;

  // get all validation errors
  let errors = polygon.validation_errors();
  // error implements Display for human readable explanations
  println!("{}", errors[0]);
  ```
- Polygons returned by Boolean Ops are now oriented correctly (ccw shell, cw inner rings)
  - <https://github.com/georust/geo/pull/1310>
- Update `i_overlay`, which is used by the `BoolOps` trait.
  - <https://github.com/georust/geo/pull/1314>
- BREAKING: Speed up `Relate` for `PreparedGeometry` - this did require
  changing some trait constraints, but they are unlikely to affect you in
  practice unless you have your own Relate implementation.
  - <https://github.com/georust/geo/pull/1317>
- Add: PreparedGeometry::geometry and into_geometry to get at the inner geometry type.
  - <https://github.com/georust/geo/pull/1318>
- Add: Clone and Debug implementations for PreparedGeometry.
  - <https://github.com/georust/geo/pull/1324>

## 0.29.3 - 2024.12.03

- Fix crash in `BoolOps` by updating `i_overlay` to 1.9.0.
  - <https://github.com/georust/geo/pull/1275>

## 0.29.2 - 2024.11.15

- Pin `i_overlay` to < 1.8.0 to work around [recursion bug](https://github.com/georust/geo/issues/1270).
  - <https://github.com/georust/geo/pull/1271>
- Add multithreading support to `Multi*` geometries
  - <https://github.com/georust/geo/pull/1265>

## 0.29.1 - 2024.11.01

- Allow configuring of the `i_overlay` Rayon transitive dependency with a new Cargo `multithreading` flag.
  - <https://github.com/georust/geo/pull/1250>
- Improve handling of InterploatePoint with collapsed Line
  - <https://github.com/georust/geo/pull/1248>

## 0.29.0 - 2024.10.30

- Implement getter methods on `AffineTransform` to access internal elements.
  - <https://github.com/georust/geo/pull/1159>
- Fix issue in Debug impl for AffineTransform where yoff is shown instead of xoff
  - <https://github.com/georust/geo/pull/1191>
- `Polygon` in `Rect` performance improvements.
  - <https://github.com/georust/geo/pull/1192>
- Fix `AffineTransform::compose` ordering to be conventional - such that the argument is applied _after_ self.
  - <https://github.com/georust/geo/pull/1196>
- Add `PreparedGeometry` to speed up repeated `Relate` operations.
  - <https://github.com/georust/geo/pull/1197>
- Implement Frechet distance using linear algorithm to avoid `fatal runtime error: stack overflow` and improve overall performances.
  - <https://github.com/georust/geo/pull/1199>
- Bump `geo` MSRV to 1.74 and update CI
  - <https://github.com/georust/geo/pull/1201>
- Add `StitchTriangles` trait which implements a new kind of combining algorithm for `Triangle`s
  - <https://github.com/georust/geo/pull/1087>
- BREAKING: Remove deprecated `Bearing` trait
- Unify various line measurements under new `line_measures::{Bearing, Distance, Destination, InterpolatePoint}` traits
  Before:

  ```
  use geo::{GeodesicBearing, HaversineBearing, GeodesicDistance, HaversineDistance, EuclideanDistance};
  p1.geodesic_bearing(p2)
  p1.haversine_bearing(p2)
  p1.geodesic_distance(p2)
  p1.haversine_distance(p2)
  p1.euclidean_distance(p2)
  ```

  After:

  ```
  use geo::{Geodesic, Haversine, Euclidean, Bearing, Distance};
  Geodesic::bearing(p1, p2)
  Haversine::bearing(p1, p2)
  Geodesic::distance(p1, p2)
  Haversine::distance(p1, p2)
  Euclidean::distance(p1, p2)
  ```

  - <https://github.com/georust/geo/pull/1216>

- Deprecated legacy line measure traits in favor of those added in the previous changelog entry:
  - `GeodesicBearing`, `GeodesicDistance`, `GeodesicDestination`, `GeodesicIntermediate`
  - `RhumbBearing`, `RhumbDistance`, `RhumbDestination`, `RhumbIntermediate`
  - `HaversineBearing`, `HaversineDistance`, `HaversineDestination`, `HaversineIntermediate`
  - `EuclideanDistance`
  - <https://github.com/georust/geo/pull/1222>
  - <https://github.com/georust/geo/pull/1232>
- Deprecated `HaversineLength`, `EuclideanLength`, `RhumbLength`, `GeodesicLength` in favor of new generic `Length` trait.
  ```
  // Before
  line_string.euclidean_length();
  line_string.haversine_length();
  // After
  line_string.length::<Euclidean>();
  line_string.length::<Haversine>();
  ```
  - <https://github.com/georust/geo/pull/1228>
- Deprecated `DensifyHaversine`
- BREAKING: `Densify::densify` is no longer strictly Euclidean, and now accepts a generic line measure parameter.

  ```
  // Before
  line_string.densify();
  line_string.densify_haversine();
  // After
  line_string.densify::<Euclidean>();
  line_string.densify::<Haversine>();

  // Additional measures are now supported
  line_string.densify::<Geodesic>();
  line_string.densify::<Rhumb>();
  ```

- Added `InterpolatePoint::point_at_distance_between` for line_measures.
  - <https://github.com/georust/geo/pull/1235>
- Change IntersectionMatrix::is_equal_topo to now consider empty geometries as equal.
  - <https://github.com/georust/geo/pull/1223>
- Fix `(LINESTRING EMPTY).contains(LINESTRING EMPTY)` and `(MULTIPOLYGON EMPTY).contains(MULTIPOINT EMPTY)` which previously
  reported true
  - <https://github.com/georust/geo/pull/1227>
- Improve `HasDimensions::dimensions` to handle dimensionally collapsed and empty geometries more consistently.
  A collection (like MultiPolygon) will now have EmptyDimensions when all of its elements have EmptyDimensions.
  - <https://github.com/georust/geo/pull/1226>
- Enable i128 geometry types
  - <https://github.com/georust/geo/pull/1230>

## 0.28.0

- BREAKING: The `HasKernel` trait was removed and it's functionality was merged
  into `GeoNum`. If you are using common scalars for your geometry (f32, f64,
  i64, i32, i16, isize), this should have no effect on you. If you are using an
  exotic scalar type, you'll need to implement `GeoNum` for it instead of
  `HasKernel`. If you had functionality defined in terms of `HasKernel` before,
  define it in terms of `GeoNum` instead.
  - <https://github.com/georust/geo/pull/1134>
- BREAKING: Added a new `total_cmp` method to `GeoNum`. This avoids some
  potential crashes when working with geometries that contain NaN points. This
  shouldn't break for any common numeric types, but if you are using something
  exotic you'll need to manually implement `GeoNum` for your numeric type.
  - <https://github.com/georust/geo/pull/1134>
- POSSIBLY BREAKING: `SimplifyVwPreserve` trait implementation moved from
  `geo_types::CoordNum` to `geo::GeoNum` as a consequence of introducing the
  `GeoNum::total_cmp`. This shouldn't break anything for common numeric
  types, but if you are using something exotic you'll need to manually
  implement `GeoNum` for your numeric type.
- Implement ChaikinSmoothing to work on Geometry types
  - <https://github.com/georust/geo/pull/1116>
- Fix a panic when calculating the haversine closest point to a point intersecting the geometry
  - <https://github.com/georust/geo/pull/1119>
- Add `LineStringSegmentizeHaversine` trait as a an alternative to `LineStringSegmentize` for geographic coordinates.
  - <https://github.com/georust/geo/pull/1107>
- Make `SpadeTriangulationConfig` actually configurable
  - <https://github.com/georust/geo/pull/1123>
- PERF: small improvements to TriangulateSpade trait
  - <https://github.com/georust/geo/pull/1122>
- POSSIBLY BREAKING: Minimum supported version of Rust (MSRV) is now 1.70
  - <https://github.com/georust/geo/pull/1134>
- Add topological equality comparison method:
  - <https://github.com/georust/geo/pull/1133>
- Add docs to Relate trait
  - <https://github.com/georust/geo/pull/1135>
- Add remaining Relate predicates
  - <https://github.com/georust/geo/pull/1136>
- Update rstar to v0.12.0
- Implement `CoordsIter` for arrays and slices. This is useful when you'd like to use traits
  implemented for `CoordsIter` without re-allocating (e.g., creating a `MultiPoint`).
- Add `compose_many` method to `AffineOps`
  - <https://github.com/georust/geo/pull/1148>
- Point in `Triangle` and `Rect` performance improvemnets
  - <https://github.com/georust/geo/pull/1057>
- Fix crashes in `BooleanOps`
  - <https://github.com/georust/geo/pull/1234>

## 0.27.0

- Use `CachedEnvelope` in R-Trees when computing euclidean distance between polygons
  - <https://github.com/georust/geo/pull/1093>
- Add an `inverse` method to `AffineTransform`
  - <https://github.com/georust/geo/pull/1092>
- Fix `Densify` trait to avoid panic with empty line string.
  - <https://github.com/georust/geo/pull/1082>
- Add `DensifyHaversine` trait to densify spherical line geometry.
  - <https://github.com/georust/geo/pull/1081>
- Add `LineStringSegmentize` trait to split a single `LineString` into `n` `LineStrings` as a `MultiLineString`.
  - <https://github.com/georust/geo/pull/1055>
- Add `EuclideanDistance` implementations for all remaining geometries.
  - <https://github.com/georust/geo/pull/1029>
- Add `HausdorffDistance` algorithm trait to calculate the Hausdorff distance between any two geometries.
  - <https://github.com/georust/geo/pull/1041>
- Add `matches` method to IntersectionMatrix for ergonomic de-9im comparisons.
  - <https://github.com/georust/geo/pull/1043>
- Simplify `CoordsIter` and `MinimumRotatedRect` `trait`s with GATs by removing an unneeded trait lifetime.
  - <https://github.com/georust/geo/pull/908>
- Add `ToDegrees` and `ToRadians` traits.
  - <https://github.com/georust/geo/pull/1070>
- Add rhumb-line operations analogous to several current haversine operations: `RhumbBearing`, `RhumbDestination`, `RhumbDistance`, `RhumbIntermediate`, `RhumbLength`.
  - <https://github.com/georust/geo/pull/1090>
- Fix coordinate wrapping in `HaversineDestination`
  - <https://github.com/georust/geo/pull/1091>
- Add `wkt!` macro to define geometries at compile time.
  - <https://github.com/georust/geo/pull/1063>
- Add `TriangulateSpade` trait which provides (un)constrained Delaunay Triangulations for all `geo_types` via the `spade` crate
  - <https://github.com/georust/geo/pull/1083>
- Add `len()` and `is_empty()` to `MultiPoint`
  - <https://github.com/georust/geo/pull/1109>

## 0.26.0

- Implement "Closest Point" from a `Point` on a `Geometry` using spherical geometry. <https://github.com/georust/geo/pull/958>
- Bump CI containers to use libproj 9.2.1
- **BREAKING**: Bump rstar and robust dependencies
  <https://github.com/georust/geo/pull/1030>

## 0.25.1

- Add `TriangulateEarcut` algorithm trait to triangulate polygons with the earcut algorithm.
  - <https://github.com/georust/geo/pull/1007>
- Add `Vector2DOps` trait to algorithms module and implemented it for `Coord<T::CoordFloat>`

  - <https://github.com/georust/geo/pull/1025>

- Add a fast point-in-polygon query datastructure that pre-processes a `Polygon` as a set of monotone polygons. Ref. `crate::algorithm::MonotonicPolygons`.
  - <https://github.com/georust/geo/pull/1018>

## 0.25.0

- Added `CrossTrackDistance` trait to calculate the distance from a point
  to the nearest point on a line
  - <https://github.com/georust/geo/pull/961>
- Performance improvements for CoordinatePosition
  - <https://github.com/georust/geo/pull/1004>
- BREAKING: Remove deprecated methods
  - <https://github.com/georust/geo/pull/1012>
  - Instead of `map_coords_inplace` use `map_coords_in_place`
  - Instead of `RotatePoint` use `Rotate`
  - Instead of `Translate#translate_inplace` use `Translate#translate_mut`

## 0.24.1

- Rename Bearing::bearing to HaversineBearing::haversine_bearing to clarify it uses great circle calculations.
  - <https://github.com/georust/geo/pull/999>
- Speed up intersection checks
  - <https://github.com/georust/geo/pull/994>
- FIX: Simplify no longer skips simplifying minimally sized Polygons and LineString
  - <https://github.com/georust/geo/pull/996>

## 0.24.0

- BREAKING: Make `SimplifyVw` naming consistent
  - <https://github.com/georust/geo/pull/957>
- Update the `Polygon` implementation of the `Simplify` algorithm to always return `Polygon`s with at least four coordinates.
  - <https://github.com/georust/geo/pull/943>
- BREAKING: Update to float_next_after-1.0.0
  <https://github.com/georust/geo/pull/952>
- POSSIBLY BREAKING: Minimum supported version of Rust (MSRV) is now 1.63
- BREAKING: Update `rstar` dependency to `0.10.0` and enable `use-rstar_0_10` feature for `geo-types.
  <https://github.com/georust/geo/pull/987>
- Added `MinimumRotatedRect` trait to calculate the MBR of geometry
  <https://github.com/georust/geo/pull/959>
- Added `GeodesicArea` trait to support geodesic area and perimeter calculations from `geographlib-rs`
  <https://github.com/georust/geo/pull/988>
- Added `GeodesicDestination` trait to support geodesic destination calculations
  <https://github.com/georust/geo/pull/991>
- Added `GeodesicBearing` trait to support geodesic bearing calculations
  <https://github.com/georust/geo/pull/991>

## 0.23.1

- Update to geo-types-0.7.8 which deprecated `Coordinate` in favor of `Coord`.
  <https://github.com/georust/geo/pull/924>
- Added doc for Transform trait to root docs index.
- Fixed an issues where calculating the convex hull of 3 collinear points
  would include all 3.
  - <https://github.com/georust/geo/pull/907>
- Added outlier detection algorithm using [LOF](https://en.wikipedia.org/wiki/Local_outlier_factor)
  - <https://github.com/georust/geo/pull/904>
- Changed license field to [SPDX 2.1 license expression](https://spdx.dev/spdx-specification-21-web-version/#h.jxpfx0ykyb60)
  - <https://github.com/georust/geo/pull/928>
- Added `RemoveRepeatedPoints` trait allowing the removal of (consecutive)
  repeated points.
- Remove polygon-polygon fast path due to ongoing lack of reliability
  - <https://github.com/georust/geo/pull/920>
- Fix RDP recursion bug
  - <https://github.com/georust/geo/pull/941>
- Clarify documentation of bearing on the HaversineDestination

## 0.23.0

- Added `AffineOps`, `Scale`, and `Skew` traits allowing the definition and
  composition of 2-D affine transforms.
  - Related Cleanup:
    - Existing `Rotate` and `Translate` traits leverage this new `AffineOps`
      trait.
    - Moved `RotatePoint::rotate_around_point` method onto
    - `Rotate::rotate_around_point` and removed `RotatePoint` trait.
    - Removed deprecated `Rotate::rotate` method, use
      `Rotate::rotate_around_center` or `Rotate::roate_around_centroid`
      instead.
    - Deprecated `Translate::translate_in_place` in favor of
      `Translate::translate_mut` to line up with naming elsewhere in the crate.
  - Implemented across several PR's:
    - <https://github.com/georust/geo/pull/866>
    - <https://github.com/georust/geo/pull/871>
    - <https://github.com/georust/geo/pull/872>
- Added `BooleanOps::clip` to clip a 1-D geometry with a 2-D geometry.
  - <https://github.com/georust/geo/pull/886>
- Added `InteriorPoint` trait to calculate a representative point inside a
  `Geometry`.
  - <https://github.com/georust/geo/pull/870>
- Added `Within` trait to determine if one Geometry is completely within
  another.
  - <https://github.com/georust/geo/pull/884>
- Added `ConvexHull` implementation for all remaining geometries.
  - <https://github.com/georust/geo/pull/889>
- Added `Contains` implementation for all remaining geometries.
  - <https://github.com/georust/geo/pull/880>
- Removed deprecated `ToGeo` trait. Use `std::convert::TryFrom<$geometry>`
  instead.
  - <https://github.com/georust/geo/pull/892>

## 0.22.1

- Fix some floating point issues with `BoolOps`
  - <https://github.com/georust/geo/pull/869>

## 0.22.0

- Add densification algorithm for linear geometry components
  - <https://github.com/georust/geo/pull/847>
- You may now specify `Geometry` rather than `Geometry<f64>` since we've added
  a default trait implementation. You may still explicitly declare the numeric
  type as f64, or any other implementation of `CoordNum`, but this should save
  you some typing if you're using f64. The same change applies to `Coordinates`
  and all the geometry variants, like `Point`, `LineString`, etc.
  - <https://github.com/georust/geo/pull/832>
- Fix fast path euclidean distance
  - <https://github.com/georust/pull/848>
- Reexport everything from the `proj` crate
  - <https://github.com/georust/geo/pull/839>
- Added a `geometry` module which re-exports all the inner geometry variants, so you
  can `use geo::geometry::*` to concisely include `Point`, `LineString`, etc.
  - <https://github.com/georust/geo/pull/853>
- Use robust predicates everywhere in geo
  - <https://github.com/georust/geo/pull/852>
- `Winding` trait is rexported under geo::algorithm::Winding (and thus
  geo::Winding and geo::prelude::Winding)
  - <https://github.com/georust/geo/pull/855/files>
- BREAKING: de-exported `WindingOrder` from `geo::WindingOrder`/`geo::algorithms::WindingOrder`.
  Instead, go back to `use geo::winding_order::WindingOrder` - it was briefly rexported as
  `geo::WindingOrder` and `geo::algorithms::WindingOrder`.
  - <https://github.com/georust/geo/pull/855/files>

## 0.21.0

- Boolean operations for `Polygon`s and `MultiPolygon`s: intersect, union, xor,
  and difference. Refer trait `bool_ops::BooleanOps` for more info.
- POSSIBLY BREAKING: Minimum supported version of Rust (MSRV) is now 1.58
- BREAKING: rstar version upgraded to 0.9.x

  - <https://github.com/georust/geo/pull/835>

- POSSIBLY BREAKING: `GeoFloat` types must now implement `num_traits::Signed` and `num_traits::Bounded`. This shouldn't
  affect you if you are using a standard `Geometry<f64>` or `Geometry<f32>` or `geo::GeoFloat` generically.
- Speed up `Relate` and `Contains` traits for large `LineStrings` and `Polygons` by using an RTree to more efficiently
  inspect edges in our topology graph.
- Flatten algorithm namespace. For example:

  ```rust
  # Before
  use geo::algorithm::area::Area;
  use geo::algorithm::bounding_rect::BoundingRect;
  # After
  use geo::{Area, BoundingRect};
  ```

- Speed up `intersects` checks by using a preliminary bbox check
  - <https://github.com/georust/geo/pull/828>
- BREAKING: Remove unneeded reference for `*MapCoords*` closure parameter.
  - <https://github.com/georust/geo/pull/810>
- BREAKING: Bump `proj` dependency to 0.26 which uses PROJ version 9.0
  - <https://github.com/georust/geo/pull/813>
- rename `Translate::translate_inplace` -> `Translate::translate_in_place`
  - <https://github.com/georust/geo/pull/811>
- `MapCoords` restructuring: <https://github.com/georust/geo/pull/811>
  - rename `MapCoordsInplace::map_coords_inplace` -> `MapCoordsInPlace::map_coords_in_place`
  - rename `TryMapCoordsInplace::try_map_coords_inplace` -> `TryMapCoordsInPlace::try_map_coords_in_place`
  - Consolidate traits `TryMapCoords` into `MapCoords` and `TryMapCoordsInplace` into `MapCoordsInPlace`
- Implement `ChamberlainDuquetteArea` for all geo types.
  - <https://github.com/georust/geo/pull/833>
- Add `{Convert,TryConvert}` traits for coordinate value type conversion.
  - <https://github.com/georust/geo/pull/836>
- BREAKING: `MapCoords`/`MapCoordsInPlace` now map `Coordinate`s rather than `(x,y)` tuples
  - <https://github.com/georust/geo/pull/837>
- Tidy fast-path distance algorithm
  - <https://github.com/georust/geo/pull/754>

## 0.20.1

- FIX: update to proper minimum geo-types version
  - <https://github.com/georust/geo/pull/815>

## 0.20.0

- Add `LinesIter` algorithm to iterate over the lines in geometries.
  - Very similar to `CoordsIter`, but only implemented where it makes sense (e.g., for `Polygon`, `Rect`, but not `Point`).
  - <https://github.com/georust/geo/pull/757>
- Add `TryMapCoordsInplace` algorithm that is similar to `TryMapCoords` but modifies a geometry in-place
  - <https://github.com/georust/geo/pull/800>

## 0.19.0

- Bump `proj` crate to 0.25.0, using PROJ 8.1.0
  - <https://github.com/georust/geo/pull/661>
  - <https://github.com/georust/geo/pull/718>
- Add `ChaikinSmoothing` algorithm
  - <https://github.com/georust/geo/pull/648>
- Fix `rotate` for multipolygons to rotate around the collection's centroid, instead of rotating each individual polygon around its own centroid.
  - <https://github.com/georust/geo/pull/651>
- Add `KNearestConcaveHull` algorithm
  - <https://github.com/georust/geo/pull/635>
- Remove cargo-tarpaulin due to instability (#676, #677)
- Fix: `ClosestPoint` for Polygon's handling of internal points
  - <https://github.com/georust/geo/pull/679>
- Implemented `ClosestPoint` method for types Triangle, Rect, GeometryCollection, Coordinate and the Geometry enum.
  - <https://github.com/georust/geo/pull/675>
- BREAKING: `TryMapCoords` Result is now generic rather than a Box<dyn Error>.
  - <https://github.com/georust/geo/issues/722>
- Add `Transform` algorithm
  - <https://github.com/georust/geo/pull/718>
- Add missing `Intersects` implementations
  - <https://github.com/georust/geo/pull/725>
- Note: The MSRV when installing the latest dependencies has increased to 1.55
  - <https://github.com/georust/geo/pull/726>
- Add `get()` to `IntersectionMatrix` for directly querying DE-9IM matrices
  - <https://github.com/georust/geo/pull/714>

## 0.18.0

- Add `line_intersection` to compute point or segment intersection of two Lines.
  - <https://github.com/georust/geo/pull/636>
- Add `Relate` trait to topologically relate two geometries based on [DE-9IM](https://en.wikipedia.org/wiki/DE-9IM) semantics.
  - <https://github.com/georust/geo/pull/639>
- Fix `Contains` implementation for Polygons to match the OGC spec using the new `Relate` trait
  - <https://github.com/georust/geo/pull/639>
- BREAKING: `Contains` no longer supports Integer `Polygon` and `Geometry`. This was a trade-off for a `Contains` implementation that was more correct for Floats.
  - <https://github.com/georust/geo/pull/639>

## 0.17.1

- Rewrite the crate documentation
  - <https://github.com/georust/geo/pull/619>
- Fix `Centroid` algorithm for `MultiLineString` when all members have only one
  point.
  - <https://github.com/georust/geo/pull/629>
- Implement `Centroid` algorithm on `Geometry` and its remaining variants.
  - <https://github.com/georust/geo/pull/629>
- Add `GeodesicIntermediate` algorithm
  - <https://github.com/georust/geo/pull/608>

## 0.17.0

- BREAKING: update geo-types to 0.7
  - <https://github.com/georust/geo/blob/geo-types-0.7.0/geo-types/CHANGES.md>
- Introduce `coords_count` method on `CoordsIter`.
  - <https://github.com/georust/geo/pull/563>
- Fix non-empty MultiPoint has 0-dimensions, not 1.
  - <https://github.com/georust/geo/pull/561>
- Add new `EuclideanDistance` implementations: `impl EuclideanDistance<Coordinate<T>> for Line`, `impl EuclideanDistance<Line> for Coordinate`, `impl EuclideanDistance<Coordinate> for Coordinate`
  - <https://github.com/georust/geo/pull/580>
- Introduce `geo::GeoFloat` and `geo::GeoNum` trait so external crates can implement methods which
  operate on geometries generically.
  - <https://github.com/georust/geo/pull/583>
  - <https://github.com/georust/geo/pull/602>
- Make `HasKernel` public to allow geo on exotic numeric types.
  - <https://github.com/georust/geo/pull/583>
- Fix panic when `simplify` is given a negative epsilon
  - <https://github.com/georust/geo/pull/584>
- Performance improvements to `simplify`
  - <https://github.com/georust/geo/pull/584>
- BREAKING: The `T` generic parameter for `CoordsIter` is now an associated type
  - <https://github.com/georust/geo/pull/593>
- Add `CoordsIter::exterior_coords_iter` method to iterate over exterior coordinates of a geometry
  - <https://github.com/georust/geo/pull/594>
- BREAKING: The `ExtremeIndices` and `ExtremePoints` traits have been combined into a new `Extremes` trait containing an `extremes` method. The output of the `extremes` method contains both indices and coordinates. The new implementation is based on `CoordsIter` instead of `ConvexHull`, and now runs 6x faster.
  - <https://github.com/georust/geo/pull/592>

## 0.16.0

- Fix panic when `simplify` is given a negative epsilon
  - <https://github.com/georust/geo/pull/537>
- Add `CoordsIter` trait for iterating over coordinates in geometries.
  - <https://github.com/georust/geo/pull/164>
- Fix edge case handling in `Contains`
  - <https://github.com/georust/geo/pull/526>
- Fix edge case handling in `line_locate_point`
  - <https://github.com/georust/geo/pull/520>
- Add `proj-network` feature enables network grid for optional `proj` integration.
  - <https://github.com/georust/geo/pull/506>
- Add `HasDimensions` trait for working with Geometry dimensionality
  - <https://github.com/georust/geo/pull/524>

## 0.15.0

- Add `Intersects` implementations for all pairs of types
  - <https://github.com/georust/geo/pull/516>
  - <https://github.com/georust/geo/pull/514>
- Add `ConcaveHull` algorithm
  - <https://github.com/georust/geo/pull/480>
- Add robust predicates
  - <https://github.com/georust/geo/pull/511>
  - <https://github.com/georust/geo/pull/505>
  - <https://github.com/georust/geo/pull/504>
  - <https://github.com/georust/geo/pull/502>
- Improve numerical stability in centroid computation
  - <https://github.com/georust/geo/pull/510>

## 0.14.2

- Bump proj version to 0.20.3
  - <https://github.com/georust/geo/pull/496>
- Change closure for `exterior_mut()` and `interiors_mut()` to be `FnOnce`
  - <https://github.com/georust/geo/pull/479>
- Bump proj version to 0.20.0 (superseded by 0.20.3)
  - <https://github.com/georust/geo/pull/472>
- Fix numerical stability in area computation
  - <https://github.com/georust/geo/pull/482>
- Fix `contains` for degenerate zero-area triangles
  - <https://github.com/georust/geo/pull/474>
- Allow MapCoords on Rect to invert coords
  - <https://github.com/georust/geo/pull/490>
- Centroid impl for MultiLineString
  - <https://github.com/georust/geo/pull/485>
- Fix Area logic for Polygon with interiors
  - <https://github.com/georust/geo/pull/487>

## 0.14.1

- Fix bug in Line-Polygon Euclidean distance
  - <https://github.com/georust/geo/pull/477>

## 0.14.0

- Bump geo-types version to 0.6.0
- Bump rstar version to 0.8.0
  - <https://github.com/georust/geo/pull/468>
- Bump proj version to 16.2
  - <https://github.com/georust/geo/pull/453>
- Extract PostGIS integration out to new `geo-postgis` crate
  - <https://github.com/georust/geo/pull/466>
- Add new `GeodesicDistance` and `GeodesicLength` algorithms
  - <https://github.com/georust/geo/pull/440>
- Implement `Area` for all types
  - <https://github.com/georust/geo/pull/459>
- Implement `BoundingRect` for all types
  - <https://github.com/georust/geo/pull/443>
- Add more `Contains` implementations
  - <https://github.com/georust/geo/pull/451>
- Fix Vincenty algorithms for equatorial and coincident points
  - <https://github.com/georust/geo/pull/438>
- Separate area algorithms into unsigned and signed methods. For clarity, the existing `Area#area`, which can return a negative value depending on winding order, has been renamed to `Area#signed_area`. Most likely, if you aren't sure which one to use, you'll want `unsigned_area` which is always positive.
  - <https://github.com/georust/geo/pull/463>

## 0.13.0

- Bump geo-types dependency to 0.5.0
- Bump proj dependency to 0.15.1
- Add a mutable Coordinate iterator to LineString
  - <https://github.com/georust/geo/pull/404>
- Fix for rectangle intersection check
  - <https://github.com/georust/geo/pull/420>
- Bump proj to 0.14.4
  - <https://github.com/georust/geo/pull/412>
- Add `BoundingRect` implementation for `Rect`
  - <https://github.com/georust/geo/pull/355>
- Add Chamberlain–Duquette area algorithm
  - <https://github.com/georust/geo/pull/369>
- Make Euclidean Line-Line distance symmetrical
  - <https://github.com/georust/geo/pull/371>
- Bump rstar dependency to 0.4
  - <https://github.com/georust/geo/pull/373>
- Mark `ToGeo` as deprecated
  - <https://github.com/georust/geo/pull/375>
- Remove usages of 'failure' crate
  - <https://github.com/georust/geo/pull/388>

## 0.12.2

- Introduce `point!`, `line_string!`, and `polygon!` macros.
  - <https://github.com/georust/geo/pull/352>

## 0.12.1

- Add `FrechetDistance` algorithm
  - <https://github.com/georust/geo/pull/348>

## 0.12.0

- Bump `geo-types` dependency to 0.4.0
- Bump `rstar` and `proj` dependencies
  - <https://github.com/georust/geo/pull/346>
- Implement `Centroid` for `MultiPoint`
  - <https://github.com/georust/geo/pull/322>

## 0.11.0

- Replace the [spade](https://crates.io/crates/spade) crate with the [rstar](https://crates.io/crates/rstar) crate
  - <https://github.com/georust/geo/pull/314>
- Remove unnecessary algorithm trait bounds
  - <https://github.com/georust/geo/pull/320/>

## 0.10.3

- Add `MapCoords` for `Rect`s
  - <https://github.com/georust/geo/commit/11e4b67ae5fa658bd556eea96ba6fd49f32921c4>
- Rewrite vincenty/haversine docs; specify param/return units.
  - <https://github.com/georust/geo/commit/6ca45c347c53c5f0fd41b90ff5d0ba67d1b2ec15>
- `Area` can work on some non-`Float` geometries (e.g. `Rect<Integer>`)
  - <https://github.com/georust/geo/commit/1efd87a9bf3f4140f252014b59ff174af8e014aa>

## 0.10.2

- Add `to_degrees` and `to_radians` methods on `Point`s
  - <https://github.com/georust/geo/pull/306>

## 0.10.1

- Fix some edge case on centroid computation
  - <https://github.com/georust/geo/pull/305>

## 0.10.0

- Remove unnecessary borrows in function params for `Copy` types.
  - <https://github.com/georust/geo/pull/265>
- Rename bounding ‘box’ to ‘rect’; move structure to geo-types.
  - <https://github.com/georust/geo/pull/295>

## 0.9.1

- Fix Line-Polygon euclidean distance
  - <https://github.com/georust/geo/pull/226>
- Implement `EuclideanDistance` for `MultiPolygon` to `Line` and `Line` to `MultiPolygon`
  - <https://github.com/georust/geo/pull/227>
- Add `Line`-`LineString` euclidean distance
  - <https://github.com/georust/geo/pull/232>
- Add `VincentyDistance` and `VincentyLength` algorithms
  - <https://github.com/georust/geo/pull/213>
- Add `HaversineIntermediate` algorithm

## 0.9.0

- Make serde an optional dependency for `geo`, rename feature to `use-serde`
  - <https://github.com/georust/geo/pull/209>
- Use the `proj` crate, rename feature to `use-proj`
  - <https://github.com/georust/geo/pull/214>
- Return unboxed iterators from `LineString::lines`, `Winding::points_cw`, and `Winding::points_ccw`
  - <https://github.com/georust/geo/pull/218>
- Fix compilation errors when using the `proj` feature
  - <https://github.com/georust/geo/commit/0924f3179c95bfffb847562ee91675d7aa8454f5>
- Add `Polygon`-`Polygon` and `LineString`-`LineString` distance
  - <https://github.com/georust/geo/pull/219>
- Update postgis optional dependency to 0.6
  - <https://github.com/georust/geo/pull/215>
- Clarify wording for Contains algorithm.
  - <https://github.com/georust/geo/pull/220>

## 0.8.3

- Reexport core types from `geo-types`
  - <https://github.com/georust/geo/pull/201>

## 0.8.2

- Fix documentation generation on docs.rs
  - <https://github.com/georust/geo/pull/202>

## 0.8.1

- Fix centroid calculation for degenerate polygons
  - <https://github.com/georust/geo/pull/203>

## 0.8.0

- Prefix Euclidean distance/length traits with 'Euclidean'.
  - <https://github.com/georust/geo/pull/200>
- Bump num-traits: 0.1 → 0.2
  - <https://github.com/georust/geo/pull/188>
- Implement `SpatialObject` for `Line` type
  - <https://github.com/georust/geo/pull/181>
- Implement a `TryMapCoords` trait
  - <https://github.com/georust/geo/pull/191>
  - <https://github.com/georust/geo/pull/197>
- Impl Polygon convexity function on the type
  - <https://github.com/georust/geo/pull/195>
- Implement rust-proj as an optional feature within geo
  - <https://github.com/georust/geo/pull/192>

## 0.7.4

- [`cross_prod` method added to `Point`](https://github.com/georust/geo/pull/189)

## 0.7.3

- [Allow coordinates to be more types (not just `Float`s)](https://github.com/georust/geo/pull/187)

## 0.7.2

- [Easy methods to convert a Geometry to the underlying type](https://github.com/georust/geo/pull/184)
- [Map coords inplace](https://github.com/georust/geo/pull/170)
- [Added bearing trait]https://github.com/georust/geo/pull/186)
- [Winding/Orientation for LineStrings](https://github.com/georust/geo/pull/169)

## 0.7.1

- [Add Haversine length algorithm](https://github.com/georust/geo/pull/183)

## 0.7.0

- [Add `Line` to the `Geometry` `enum`](https://github.com/georust/geo/pull/179)
- [Use new bulk-load method for initial R\* Tree population](https://github.com/georust/geo/pull/178)
- [Add PostGIS and GeoJSON integration/conversions](https://github.com/georust/geo/pull/180)

## 0.6.3

- [Initial implementation of a `ClosestPoint` algorithm](https://github.com/georust/geo/pull/167)

## 0.6.2

- [Add a prelude: `use geo::prelude::*`](https://github.com/georust/geo/pull/162)

## 0.6.1

- [Add a `lines` iterator method on `LineString`](https://github.com/georust/geo/pull/160)
- [Implement `Contains<Polygon>` for `Polygon`](https://github.com/georust/geo/pull/159)
- [Correctly check for LineString containment in Polygon](https://github.com/georust/geo/pull/158)

## 0.6.0

- [Remove unnecessary trait bound on `Translate`](https://github.com/georust/geo/pull/148)
- [Topology preserving Visvalingam-Whyatt algorithm](https://github.com/georust/geo/pull/143)
- [Implement `Copy` for `Line`](https://github.com/georust/geo/pull/150)
- [Rewrite `RotatePoint` impls to be generic](https://github.com/georust/geo/pull/153)
- [Add associated return type for `BoundingBox`](https://github.com/georust/geo/pull/156)
- [Add associated return type for `Centroid`](https://github.com/georust/geo/pull/154)

## 0.5.0

- [Reimplement `Translate` trait using `MapCoords`](https://github.com/georust/geo/pull/145)

## 0.4.13

- [Implement Simplification traits for more types](https://github.com/georust/geo/pull/135)
- [Add a MapCoords trait](https://github.com/georust/geo/pull/136)

## 0.4.12

- [Improve robustness when calculating distance from a point to a
  line-segment](https://github.com/georust/geo/pull/139)

## 0.4.11

- [Add `From`, `IntoIterator`, `Into` impls; add doc comments](https://github.com/georust/geo/pull/131)

## 0.4.10

- [Add `Translation` trait.](https://github.com/georust/geo/pull/128)

## 0.4.9

- [Add `Into` trait implementations.](https://github.com/georust/geo/pull/129)

## 0.4.8

- [Add `HaversineDestination` algorithm trait](https://github.com/georust/geo/pull/124)

## 0.4.7

- [Serializing/deserializing via serde](https://github.com/georust/geo/pull/125)

## 0.4.6

- [Fix incorrect usage of `abs_sub`](https://github.com/georust/geo/pull/120)

## 0.4.5

- [Add `Line` type (representing a line segment)](https://github.com/georust/geo/pull/118)

## 0.4.4

- [Quickhull orientation fix](https://github.com/georust/geo/pull/110)
- [Implement distance traits for more geometries](https://github.com/georust/geo/pull/113)
- [Correctly calculate centroid for complex polygons](https://github.com/georust/geo/pull/112)
- [Add `Orient` trait for polygon](https://github.com/georust/geo/pull/108)
- [Add geometry rotation](https://github.com/georust/geo/pull/107)
- [Add extreme point-finding](https://github.com/georust/geo/pull/114)
- [Add contains point impl for bbox](https://github.com/georust/geo/commit/3e00ef94c3d69e6d0b1caab86224469ced9444e6)

## 0.4.3

- [Implement Point to multipart geometry distance methods](https://github.com/georust/geo/pull/104)
- [Fixture cleanup](https://github.com/georust/geo/pull/105)

## 0.4.2

- [Fix Haversine distance implementation bug](https://github.com/georust/geo/pull/101)

## 0.4.1

- [Implement convex hull algorithm](https://github.com/georust/geo/pull/89)

## 0.4.0

- [Implement Haversine algorithm](https://github.com/georust/geo/pull/90)
- [fix when multipolygon composed of two polygons of opposite clockwise](https://github.com/georust/geo/commits/master)
- [Migrate from 'num' to 'num_traits' crate](https://github.com/georust/geo/pull/86)

## 0.3.2

- [Add Visvalingam-Whyatt line-simplification algorithm](https://github.com/georust/geo/pull/84)

## 0.3.1

- [Within Epsilon matcher](https://github.com/georust/geo/pull/82)

## 0.3.0

- [Add named fields for the `Polygon` structure](https://github.com/georust/geo/pull/68)

## 0.2.8

- [Implement `Intersects<Bbox<T>> for Polygon`](https://github.com/georust/geo/pull/76)

## 0.2.7

- [Implement `Intersects<Polygon<T>> for Polygon`](https://github.com/georust/geo/issues/69)

## 0.2.6

- [Add Point to Polygon and Point to LineString distance methods](https://github.com/georust/geo/pull/61)

## 0.2.5

- [Implement LineString simplification](https://github.com/georust/geo/pull/55)

## 0.2.4

- [Performance improvements when iterating over pairs of coordinates](https://github.com/georust/geo/pull/50)

## 0.2.3

- [Add type Bbox and trait BoundingBox](https://github.com/georust/geo/pull/41)

## 0.2.2

- [Add the Length trait and implement Length for LineString and MultiLineString](https://github.com/georust/geo/pull/44)

## 0.2.1

- [Modify area for Polygon to consider also the isles](https://github.com/georust/geo/pull/43)
- [Add area trait to MultiPolygon](https://github.com/georust/geo/pull/43)

## 0.2.0

- [Data structures and traits are now generic (previously all were `f64`)](https://github.com/georust/geo/pull/30)
- [`geo::COORD_PRECISION` is now `f32` (previously was `f64`)](https://github.com/georust/geo/pull/40)

## 0.1.1

- [`Intersects` trait bugfixes](https://github.com/georust/geo/pull/34)

## 0.1.0

- [Add `Area` trait](https://github.com/georust/geo/pull/31)
- [Add `Contains` trait](https://github.com/georust/geo/pull/31)
- [Add `Distance` trait, remove `Point::distance_to`](https://github.com/georust/geo/pull/31)
- [Add `Intersects` trait](https://github.com/georust/geo/pull/31)
- [Implement `Centroid` trait for `MultiPolygon`](https://github.com/georust/geo/pull/31)

## 0.0.7

- [Implement `Centroid` trait, `Point::distance_to` method](https://github.com/georust/geo/pull/24)
