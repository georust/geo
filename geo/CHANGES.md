# Changes

## Unreleased

* Implement getter methods on `AffineTransform` to access internal elements.
  * <https://github.com/georust/geo/pull/1159>

## 0.28.0

* BREAKING: The `HasKernel` trait was removed and it's functionality was merged
  into `GeoNum`. If you are using common scalars for your geometry (f32, f64,
  i64, i32, i16, isize), this should have no effect on you. If you are using an
  exotic scalar type, you'll need to implement `GeoNum` for it instead of
  `HasKernel`. If you had functionality defined in terms of `HasKernel` before,
  define it in terms of `GeoNum` instead.
  * <https://github.com/georust/geo/pull/1134>
* BREAKING: Added a new `total_cmp` method to `GeoNum`. This avoids some
  potential crashes when working with geometries that contain NaN points. This
  shouldn't break for any common numeric types, but if you are using something
  exotic you'll need to manually implement `GeoNum` for your numeric type.
  * <https://github.com/georust/geo/pull/1134>
* POSSIBLY BREAKING: `SimplifyVwPreserve` trait implementation moved from
  `geo_types::CoordNum` to `geo::GeoNum` as a consequence of introducing the
  `GeoNum::total_cmp`. This shouldn't break anything for common numeric
  types, but if you are using something exotic you'll need to manually
  implement `GeoNum` for your numeric type.
* Implement ChaikinSmoothing to work on Geometry types
  * <https://github.com/georust/geo/pull/1116>
* Fix a panic when calculating the haversine closest point to a point intersecting the geometry
  * <https://github.com/georust/geo/pull/1119>
* Add `LineStringSegmentizeHaversine` trait as a an alternative to `LineStringSegmentize` for geographic coordinates.
  * <https://github.com/georust/geo/pull/1107>
* Make `SpadeTriangulationConfig` actually configurable
  * <https://github.com/georust/geo/pull/1123>
* PERF: small improvements to TriangulateSpade trait
  * <https://github.com/georust/geo/pull/1122>
* POSSIBLY BREAKING: Minimum supported version of Rust (MSRV) is now 1.70
  * <https://github.com/georust/geo/pull/1134>
* Add topological equality comparison method:
  * <https://github.com/georust/geo/pull/1133>
* Add docs to Relate trait
  * <https://github.com/georust/geo/pull/1135>
* Add remaining Relate predicates
  * <https://github.com/georust/geo/pull/1136>
* Update rstar to v0.12.0
* Implement `CoordsIter` for arrays and slices. This is useful when you'd like to use traits
  implemented for `CoordsIter` without re-allocating (e.g., creating a `MultiPoint`).
* Add `compose_many` method to `AffineOps`
  * <https://github.com/georust/geo/pull/1148>
* Point in `Triangle` and `Rect` performance improvemnets
  * <https://github.com/georust/geo/pull/1057>

## 0.27.0

* Use `CachedEnvelope` in R-Trees when computing euclidean distance between polygons
  * <https://github.com/georust/geo/pull/1093>
* Add an `inverse` method to `AffineTransform`
  * <https://github.com/georust/geo/pull/1092>
* Fix `Densify` trait to avoid panic with empty line string.
  * <https://github.com/georust/geo/pull/1082>
* Add `DensifyHaversine` trait to densify spherical line geometry.
  * <https://github.com/georust/geo/pull/1081>
* Add `LineStringSegmentize` trait to split a single `LineString` into `n` `LineStrings` as a `MultiLineString`.
  * <https://github.com/georust/geo/pull/1055>
* Add `EuclideanDistance` implementations for all remaining geometries.
  * <https://github.com/georust/geo/pull/1029>
* Add `HausdorffDistance` algorithm trait to calculate the Hausdorff distance between any two geometries.
  * <https://github.com/georust/geo/pull/1041>
* Add `matches` method to IntersectionMatrix for ergonomic de-9im comparisons.
  * <https://github.com/georust/geo/pull/1043>
* Simplify `CoordsIter` and `MinimumRotatedRect` `trait`s with GATs by removing an unneeded trait lifetime.
  * <https://github.com/georust/geo/pull/908>
* Add `ToDegrees` and `ToRadians` traits.
  * <https://github.com/georust/geo/pull/1070>
* Add rhumb-line operations analogous to several current haversine operations: `RhumbBearing`, `RhumbDestination`, `RhumbDistance`, `RhumbIntermediate`, `RhumbLength`.
  * <https://github.com/georust/geo/pull/1090>
* Fix coordinate wrapping in `HaversineDestination`
  * <https://github.com/georust/geo/pull/1091>
* Add `wkt!` macro to define geometries at compile time.
  * <https://github.com/georust/geo/pull/1063>
* Add `TriangulateSpade` trait which provides (un)constrained Delaunay Triangulations for all `geo_types` via the `spade` crate
  * <https://github.com/georust/geo/pull/1083>
* Add `len()` and `is_empty()` to `MultiPoint`
  * <https://github.com/georust/geo/pull/1109>

## 0.26.0

* Implement "Closest Point" from a `Point` on a `Geometry` using spherical geometry. <https://github.com/georust/geo/pull/958>
* Bump CI containers to use libproj 9.2.1
* **BREAKING**: Bump rstar and robust dependencies
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
  * <https://github.com/georust/geo/pull/892>

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
