# Changes

## Unrealeased

* Add `Polygon::try_exterior_mut` and `Polygon::try_interiors_mut`.
  <https://github.com/georust/geo/pull/1071>

## 0.7.11
* Bump rstar dependency
  <https://github.com/georust/geo/pull/1030>

## 0.7.10

* Implement `From<&Line>` for `LineString`

## 0.7.9

* Return `DoubleEndedIterator` from `LineString::points` and `LineString::points_mut`
  * <https://github.com/georust/geo/pull/951>
* POSSIBLY BREAKING: Minimum supported version of Rust (MSRV) is now 1.63
* Add `no_std` compatibility when the new default `std` feature is disabled
  * <https://github.com/georust/geo/pull/936>
* Support `rstar` version `0.10` in feature `use-rstar_0_10`.

## 0.7.8

* Rename `Coordinate` to `Coord`; add deprecated `Coordinate` that is an alias for `Coord`
* Pin `arbitrary` version to 1.1.3 until our MSRV catches up with its latest release 
* Add `point.x_mut()` and `point.y_mut()` methods on `Points`
* Changed license field to [SPDX 2.1 license expression](https://spdx.dev/spdx-specification-21-web-version/#h.jxpfx0ykyb60)
  * <https://github.com/georust/geo/pull/928>
* Fix typo in deprecated attribute, which will become a compiler error in a future version of rustc.
  * <https://github.com/georust/geo/pull/932>

## 0.7.7

* Fixed: using empty `polygon![]` macro no longer requires including `line_string!` macro

## 0.7.6

* You may now specify `Geometry` rather than `Geometry<f64>` since we've added
  a default trait implementation. You may still explicitly declare the numeric
  type as f64, or any other implementation of `CoordNum`, but this should save
  you some typing if you're using f64. The same change applies to `Coordinates`
  and all the geometry variants, like `Point`, `LineString`, etc.
  * <https://github.com/georust/geo/pull/832>
* the `geometry` module now re-exports all the inner geometry variants, so you
  can `use geo_types::geometry::*` to concisely include `Point`, `LineString`, etc.
  * <https://github.com/georust/geo/pull/853>

## 0.7.5

* Add `split_x` and `split_y` methods on `Rect`
  * <https://github.com/georust/geo/pull/823>
* Add support for `Polygon` in `RTree`
  * <https://github.com/georust/geo/pull/351>
* Deprecate `GeometryCollection::from(single_geom)` in favor of `GeometryCollection::from(vec![single_geom])`
  * <https://github.com/georust/geo/pull/821>

## 0.7.4

* BREAKING: Make `Rect::to_lines` return lines in winding order for `Rect::to_polygon`.
  * <https://github.com/georust/geo/pull/757>
* Note: All crates have been migrated to Rust 2021 edition. The MSRV when installing the latest dependencies has increased to 1.56.
  * <https://github.com/georust/geo/pull/741>
* Macros `coord!`, `point!`, `line_string!`, and `polygon!` now support trailing commas such as `coord! { x: 181.2, y: 51.79, }`
  * <https://github.com/georust/geo/pull/752>
* Internal cleanup: Explicitly declare `use-rstar_0_8` and `use-rstar_0_9` features to be explicit which rstar version is being used. For backward compatibility, the `use-rstar` feature will still enable `use-rstar_0_8`.
  * <https://github.com/georust/geo/pull/759>
* Add missing size_hint() method for point and coordinate iterators on LineString
  * <https://github.com/georust/geo/issues/762>
* Add ExactsizeIterator impl for Points iterator on LineString
  * <https://github.com/georust/geo/pull/767>
* Extend `point!` macro to support single coordinate expression arguments `point!(coordinate)` (coordinate can be created with the `coord!` macro)
  * <https://github.com/georust/geo/pull/775>
* `LineString`, `MultiPoint`, `MultiPolygon`, `Triangle`, `MultiLineString` now have a new constructor `new(...)`. `GeometryCollection` has a `new_from(...)` constructor. `GeometryCollection::new()` has been deprecated - use `GeometryCollection::default()` instead. Do not use tuple constructors like ~~`MultiPoint(...)`~~ for any of the geo-types. Use `MultiPoint::new(...)` and similar ones instead.
  * PRs: [MultiPolygon::new](https://github.com/georust/geo/pull/786), [MultiLineString::new](https://github.com/georust/geo/pull/784), [Triangle::new](https://github.com/georust/geo/pull/783), [GeometryCollection::new_from](https://github.com/georust/geo/pull/782), [LineString::new](https://github.com/georust/geo/pull/781), [MultiPoint::new](https://github.com/georust/geo/pull/778), [Point::from](https://github.com/georust/geo/pull/777)

## 0.7.3

* DEPRECATION: Deprecate `Point::lng`, `Point::lat`, `Point::set_lng` and `Point::set_lat`
  * <https://github.com/georust/geo/pull/711>
* Support `rstar` version `0.9` in feature `use-rstar_0_9`
  * <https://github.com/georust/geo/pull/682>
* `Geometry` and `GeometryCollection` now support serde.
  * <https://github.com/georust/geo/pull/704>
* Add `Coordinate` iterators to LineString, regularise its iterator methods, and refactor its docs
  * <https://github.com/georust/geo/pull/705>
* Add +=, -=, \*=, and /= for Point
  * <https://github.com/georust/geo/pull/715>
* Note: The MSRV when installing the latest dependencies has increased to 1.55
  * <https://github.com/georust/geo/pull/726>

## 0.7.2

* Implement `RelativeEq` and `AbsDiffEq` for fuzzy comparison of remaining Geometry Types
  * <https://github.com/georust/geo/pull/628>
* Implement `From<Line>` for `LineString`
  * <https://github.com/georust/geo/pull/634>
* Add optional `arbitrary` feature for integration with the [arbitrary](https://github.com/rust-fuzz/arbitrary) crate
  * <https://github.com/georust/geo/pull/622>

## 0.7.1

* Implement `Default` on `Coordinate` and `Point` structs (defaults to `(x: 0, y: 0)`)
  * <https://github.com/georust/geo/pull/616>
* Add specific details about conversion failures in the newly public `geo_types::Error`
  * <https://github.com/georust/geo/pull/614>

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
* Introduce `Rect::try_new` constructor which does not panic
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
* Migrate `Line` and `LineString` to be a series of `Coordinates` (not `Points`).
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
