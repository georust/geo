# Changes

## Unreleased

* Add Changes Here
* Implement Default on Coordinate struct (Defaults to (0,0)) #612

## 0.7.0

* BREAKING: `geo_types::CoordinateType` now extends Debug and has been deprecated in favor of `geo_types::CoordNum` and `geo_types::CoordFloat`
  * <https://github.com/georust/geo/pull/563>
* BREAKING: Introduce `use-rstar` feature rather than `rstar` so that `approx` dependency can be optional
  * <https://github.com/georust/geo/pull/567>
* Implement `approx::{RelativeEq, AbsDiffEq}` for geo-types when using the `approx` feature
  * <https://github.com/georust/geo/pull/567>
* `geo_types::LineString::num_coords` has been deprecated in favor of `geo::algorithm::coords_iter::CoordsIter::coords_count`
  * <https://github.com/georust/geo/pull/563>

## 0.6.2

* Add `into_iter`, `iter` and `iter_mut` methods for `MultiPolygon`, `MultiPoint`, and `MultiLineString`
  * <https://github.com/georust/geo/pull/539>
* `Rect::new` automatically determines min/max points. Deprecates `Rect::try_new` which can no longer fail.
  * <https://github.com/georust/geo/pull/519>
* Add `MultiLineString::is_closed` method
  * <https://github.com/georust/geo/pull/523>

## 0.6.1

* Add documentation on semantics (based on OGC-SFA)
  * <https://github.com/georust/geo/pull/516>
* Add vector-space operations to `Coordinate` and `Point`
  * <https://github.com/georust/geo/pull/505>

## 0.6.0

* Remove `COORD_PRECISION` which was an arbitrary constant of 0.1m
  * <https://github.com/georust/geo/pull/462>
* Bump rstar version to 0.8.0
  * <https://github.com/georust/geo/pull/468>
* Add `Triangle` and `Rect` to `Geometry`
  * <https://github.com/georust/geo/pull/432>
* Introduce `Rect::try_new` constructor which doesn’t panic
  * <https://github.com/georust/geo/pull/442>
* Add `Rect::center` method
  * <https://github.com/georust/geo/pull/450>
* Derive `Eq` for types when applicable
  * <https://github.com/georust/geo/pull/431>
  * <https://github.com/georust/geo/pull/435>
* Implement `From<Triangle> for Polygon`
  * <https://github.com/georust/geo/pull/433>

## 0.5.0

* Update Geometry enum with iterators and TryFrom impls for primitives
  * https://github.com/georust/geo/pull/410
* Make geo-types Rect fields private to force users to use constructor (breaking change)
  * <https://github.com/georust/geo/pull/374>
* Bump rstar dependency to 0.4
  * <https://github.com/georust/geo/pull/373>
* Fix link to `LineString` in docs
  * <https://github.com/georust/geo/pull/381>
* Fix typo in Rect docs about min/max positions.
  * <https://github.com/georust/geo/pull/385>
* Implement `Hash` for all types in `geo-types`
  * <https://github.com/georust/geo/pull/389>

## 0.4.3

* Introduce `point!`, `line_string!`, and `polygon!` macros.
  * <https://github.com/georust/geo/pull/352>
  * <https://github.com/georust/geo/pull/357>
* Add `Rect` constructor that enforces `min.{x,y} < max.{x,y}`
  * <https://github.com/georust/geo/pull/360>

## 0.4.2

* Add `Polygon::num_coords`
  * <https://github.com/georust/geo/pull/348>

## 0.4.1

* Add `Polygon::interiors_push` - Adds an interior ring to a `Polygon`
  * <https://github.com/georust/geo/pull/347>

## 0.4.0

* Rewrite `Polygon` structure to enforce closed `LineString` rings
  * <https://github.com/georust/geo/pull/337>
* Implement `Into<Geometry>` for `Line`
  * <https://github.com/georust/geo/pull/340>
* Implement `Index<usize>` for `LineString` to get the coordinate at that position
  * <https://github.com/georust/geo/pull/341>
* Bump `rstar` dependency
  * <https://github.com/georust/geo/pull/346>
* Ability to construct `MultiPolygon` from `Vec` of anything that implements `Into<Polygon>`
  * <https://github.com/georust/geo/pull/342>
* Add `new`, `is_empty`, `len` functions on `GeometryCollection`
  * <https://github.com/georust/geo/pull/339>
* Tweak `Geometry` method names slightly
  * <https://github.com/georust/geo/pull/343>
* Remove unnecessary references in function signatures
  * <https://github.com/georust/geo/pull/344>

## 0.3.0

* Replace the [spade](https://crates.io/crates/spade) crate with the [rstar](https://crates.io/crates/rstar) crate
  * <https://github.com/georust/geo/pull/314>
* Remove unnecessary algorithm trait bounds
  * <https://github.com/georust/geo/pull/320/>

## 0.2.2

* Fix misnamed `serde` feature flag.
  * <https://github.com/georust/geo/pull/316>
* Add `width` and `height` helpers on `Rect`.
  * <https://github.com/georust/geo/pull/317>

## 0.2.1

* Add `to_lines` method on a `Triangle`
  * <https://github.com/georust/geo/pull/313>

## 0.2.0

* Introduce `Line::{dx, dy, slope, determinant}` methods.
  * <https://github.com/georust/geo/pull/246>
* Remove unnecessary borrows in function params for `Copy` types.
  * <https://github.com/georust/geo/pull/265>
* Introduce `x_y` method on `Point` and `Coordinate`
  * <https://github.com/georust/geo/pull/277>
* Migrate `Line` aand `LineString` to be a series of `Coordinates` (not `Points`).
  * <https://github.com/georust/geo/pull/244>
* Introduce Triangle geometry type.
  * <https://github.com/georust/geo/pull/285>
* Rename bounding ‘box’ to ‘rect’; move structure to geo-types.
  * <https://github.com/georust/geo/pull/295>


## 0.1.1

* Allow LineString creation from vec of two-element CoordinateType array
  * <https://github.com/georust/geo/pull/223>


## 0.1.0

* New crate with core types from `geo`
  * <https://github.com/georust/geo/pull/201>
