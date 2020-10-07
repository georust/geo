# Changes

## Unreleased

* Add `line_intersection` to compute point or segment intersection of two Lines.
  * <https://github.com/georust/geo/pull/636>
* Add `Relate` trait to topologically relate two geometries based on [DE-9IM](https://en.wikipedia.org/wiki/DE-9IM) semantics.
  * <https://github.com/georust/geo/pull/639>
* Fix `Contains` implementation for Polygons to match the OGC spec using the new `Relate` trait
  * <https://github.com/georust/geo/pull/639>
* BREAKING: `Contains` no longer supports integer `Polygon` and `Geometry`
  * <https://github.com/georust/geo/pull/639>

## 0.17.1

* Rewrite the crate documentation
  * <https://github.com/georust/geo/pull/619>
* Fix `Centroid` algorithm for `MultiLineString` when all members have only one
  point.
  * <https://github.com/georust/geo/pull/629>
* Implement `Centroid` algorithm on `Geometry` and its remaining variants.
  * <https://github.com/georust/geo/pull/629>

## 0.17.1

* Add `GeodesicIntermediate` algorithm
  * <https://github.com/georust/geo/pull/608>

## 0.17.0

* BREAKING: update geo-types to 0.7
  * <https://github.com/georust/geo/blob/geo-types-0.7.0/geo-types/CHANGES.md>
* Introduce `coords_count` method on `CoordsIter`.
  * <https://github.com/georust/geo/pull/563>
* Fix non-empty MultiPoint has 0-dimensions, not 1.
  * <https://github.com/georust/geo/pull/561>
* Add new `EuclideanDistance` implementations: `impl EuclideanDistance<Coordinate<T>> for Line`, `impl EuclideanDistance<Line> for Coordinate`, `impl EuclideanDistance<Coordinate> for Coordinate`
  * <https://github.com/georust/geo/pull/580>
* Introduce `geo::GeoFloat` and `geo::GeoNum` trait so external crates can implement methods which
  operate on geometries generically.
  * <https://github.com/georust/geo/pull/583>
  * <https://github.com/georust/geo/pull/602>
* Make `HasKernel` public to allow geo on exotic numeric types.
  * <https://github.com/georust/geo/pull/583>
* Fix panic when `simplify` is given a negative epsilon
  * <https://github.com/georust/geo/pull/584>
* Performance improvements to `simplify`
  * <https://github.com/georust/geo/pull/584>
* BREAKING: The `T` generic parameter for `CoordsIter` is now an associated type
  * <https://github.com/georust/geo/pull/593>
* Add `CoordsIter::exterior_coords_iter` method to iterate over exterior coordinates of a geometry
  * <https://github.com/georust/geo/pull/594>
* BREAKING: The `ExtremeIndices` and `ExtremePoints` traits have been combined into a new `Extremes` trait containing an `extremes` method. The output of the `extremes` method contains both indices and coordinates. The new implementation is based on `CoordsIter` instead of `ConvexHull`, and now runs 6x faster.
  * <https://github.com/georust/geo/pull/592>

## 0.16.0

* Fix panic when `simplify` is given a negative epsilon
  * <https://github.com/georust/geo/pull/537>
* Add `CoordsIter` trait for iterating over coordinates in geometries.
  * <https://github.com/georust/geo/pull/164>
* Fix edge case handling in `Contains`
  * <https://github.com/georust/geo/pull/526>
* Fix edge case handling in `line_locate_point`
  * <https://github.com/georust/geo/pull/520>
* Add `proj-network` feature enables network grid for optional `proj` integration.
  * <https://github.com/georust/geo/pull/506>
* Add `HasDimensions` trait for working with Geometry dimensionality
  * <https://github.com/georust/geo/pull/524>

## 0.15.0

* Add `Intersects` implementations for all pairs of types
  * <https://github.com/georust/geo/pull/516>
  * <https://github.com/georust/geo/pull/514>
* Add `ConcaveHull` algorithm
  * <https://github.com/georust/geo/pull/480>
* Add robust predicates
  * <https://github.com/georust/geo/pull/511>
  * <https://github.com/georust/geo/pull/505>
  * <https://github.com/georust/geo/pull/504>
  * <https://github.com/georust/geo/pull/502>
* Improve numerical stability in centroid computation
  * <https://github.com/georust/geo/pull/510>

## 0.14.2

* Bump proj version to 0.20.3
  * <https://github.com/georust/geo/pull/496>
* Change closure for `exterior_mut()` and `interiors_mut()` to be `FnOnce`
  * <https://github.com/georust/geo/pull/479>
* Bump proj version to 0.20.0 (superseded by 0.20.3)
  * <https://github.com/georust/geo/pull/472>
* Fix numerical stability in area computation
  * <https://github.com/georust/geo/pull/482>
* Fix `contains` for degenerate zero-area triangles
  * <https://github.com/georust/geo/pull/474>
* Allow MapCoords on Rect to invert coords
  * <https://github.com/georust/geo/pull/490>
* Centroid impl for MultiLineString
  * <https://github.com/georust/geo/pull/485>
* Fix Area logic for Polygon with interiors
  * <https://github.com/georust/geo/pull/487>

## 0.14.1

* Fix bug in Line-Polygon Euclidean distance
  * <https://github.com/georust/geo/pull/477>

## 0.14.0

* Bump geo-types version to 0.6.0
* Bump rstar version to 0.8.0
  * <https://github.com/georust/geo/pull/468>
* Bump proj version to 16.2
  * <https://github.com/georust/geo/pull/453>
* Extract PostGIS integration out to new `geo-postgis` crate
  * <https://github.com/georust/geo/pull/466>
* Add new `GeodesicDistance` and `GeodesicLength` algorithms
  * <https://github.com/georust/geo/pull/440>
* Implement `Area` for all types
  * <https://github.com/georust/geo/pull/459>
* Implement `BoundingRect` for all types
  * <https://github.com/georust/geo/pull/443>
* Add more `Contains` implementations
  * <https://github.com/georust/geo/pull/451>
* Fix Vincenty algorithms for equatorial and coincident points
  * <https://github.com/georust/geo/pull/438>
* Separate area algorithms into unsigned and signed methods. For clarity, the existing `Area#area`, which can return a negative value depending on winding order, has been renamed to `Area#signed_area`.  Most likely, if you aren't sure which one to use, you'll want `unsigned_area` which is always positive.
  * <https://github.com/georust/geo/pull/463>

## 0.13.0

* Bump geo-types dependency to 0.5.0
* Bump proj dependency to 0.15.1
* Add a mutable Coordinate iterator to LineString
  * <https://github.com/georust/geo/pull/404>
* Fix for rectangle intersection check
  * <https://github.com/georust/geo/pull/420>
* Bump proj to 0.14.4
  * <https://github.com/georust/geo/pull/412>
* Add `BoundingRect` implementation for `Rect`
  * <https://github.com/georust/geo/pull/355>
* Add Chamberlain–Duquette area algorithm
  * <https://github.com/georust/geo/pull/369>
* Make Euclidean Line-Line distance symmetrical
  * <https://github.com/georust/geo/pull/371>
* Bump rstar dependency to 0.4
  * <https://github.com/georust/geo/pull/373>
* Mark `ToGeo` as deprecated
  * <https://github.com/georust/geo/pull/375>
* Remove usages of 'failure' crate
  * <https://github.com/georust/geo/pull/388>

## 0.12.2

* Introduce `point!`, `line_string!`, and `polygon!` macros.
  * <https://github.com/georust/geo/pull/352>

## 0.12.1

* Add `FrechetDistance` algorithm
  * <https://github.com/georust/geo/pull/348>

## 0.12.0

* Bump `geo-types` dependency to 0.4.0
* Bump `rstar` and `proj` dependencies
  * <https://github.com/georust/geo/pull/346>
* Implement `Centroid` for `MultiPoint`
  * <https://github.com/georust/geo/pull/322>

## 0.11.0

* Replace the [spade](https://crates.io/crates/spade) crate with the [rstar](https://crates.io/crates/rstar) crate
  * <https://github.com/georust/geo/pull/314>
* Remove unnecessary algorithm trait bounds
  * <https://github.com/georust/geo/pull/320/>

## 0.10.3

* Add `MapCoords` for `Rect`s
  * <https://github.com/georust/geo/commit/11e4b67ae5fa658bd556eea96ba6fd49f32921c4>
* Rewrite vincenty/haversine docs; specify param/return units.
  * <https://github.com/georust/geo/commit/6ca45c347c53c5f0fd41b90ff5d0ba67d1b2ec15>
* `Area` can work on some non-`Float` geometries (e.g. `Rect<Integer>`)
  * <https://github.com/georust/geo/commit/1efd87a9bf3f4140f252014b59ff174af8e014aa>

## 0.10.2

* Add `to_degrees` and `to_radians` methods on `Point`s
  * <https://github.com/georust/geo/pull/306>

## 0.10.1

* Fix some edge case on centroid computation
  * <https://github.com/georust/geo/pull/305>

## 0.10.0

* Remove unnecessary borrows in function params for `Copy` types.
  * <https://github.com/georust/geo/pull/265>
* Rename bounding ‘box’ to ‘rect’; move structure to geo-types.
  * <https://github.com/georust/geo/pull/295>

## 0.9.1

* Fix Line-Polygon euclidean distance
  * <https://github.com/georust/geo/pull/226>
* Implement `EuclideanDistance` for `MultiPolygon` to `Line` and `Line` to `MultiPolygon`
  * <https://github.com/georust/geo/pull/227>
* Add `Line`-`LineString` euclidean distance
  * <https://github.com/georust/geo/pull/232>
* Add `VincentyDistance` and `VincentyLength` algorithms
  * <https://github.com/georust/geo/pull/213>
* Add `HaversineIntermediate` algorithm

## 0.9.0

* Make serde an optional dependency for `geo`, rename feature to `use-serde`
  * <https://github.com/georust/geo/pull/209>
* Use the `proj` crate, rename feature to `use-proj`
  * <https://github.com/georust/geo/pull/214>
* Return unboxed iterators from `LineString::lines`, `Winding::points_cw`, and `Winding::points_ccw`
  * <https://github.com/georust/geo/pull/218>
* Fix compilation errors when using the `proj` feature
  * <https://github.com/georust/geo/commit/0924f3179c95bfffb847562ee91675d7aa8454f5>
* Add `Polygon`-`Polygon` and `LineString`-`LineString` distance
  * <https://github.com/georust/geo/pull/219>
* Update postgis optional dependency to 0.6
  * <https://github.com/georust/geo/pull/215>
* Clarify wording for Contains algorithm.
  * <https://github.com/georust/geo/pull/220>

## 0.8.3

* Reexport core types from `geo-types`
  * <https://github.com/georust/geo/pull/201>

## 0.8.2

* Fix documentation generation on docs.rs
  * <https://github.com/georust/geo/pull/202>

## 0.8.1

* Fix centroid calculation for degenerate polygons
  * <https://github.com/georust/geo/pull/203>

## 0.8.0

* Prefix Euclidean distance/length traits with 'Euclidean'.
  * <https://github.com/georust/geo/pull/200>
* Bump num-traits: 0.1 → 0.2
  * <https://github.com/georust/geo/pull/188>
* Implement `SpatialObject` for `Line` type
  * <https://github.com/georust/geo/pull/181>
* Implement a `TryMapCoords` trait
  * <https://github.com/georust/geo/pull/191>
  * <https://github.com/georust/geo/pull/197>
* Impl Polygon convexity function on the type
  * <https://github.com/georust/geo/pull/195>
* Implement rust-proj as an optional feature within geo
  * <https://github.com/georust/geo/pull/192>

## 0.7.4

* [`cross_prod` method added to `Point`](https://github.com/georust/geo/pull/189)

## 0.7.3

* [Allow coordinates to be more types (not just `Float`s)](https://github.com/georust/geo/pull/187)

## 0.7.2

* [Easy methods to convert a Geometry to the underlying type](https://github.com/georust/geo/pull/184)
* [Map coords inplace](https://github.com/georust/geo/pull/170)
* [Added bearing trait]https://github.com/georust/geo/pull/186)
* [Winding/Orientation for LineStrings](https://github.com/georust/geo/pull/169)

## 0.7.1

* [Add Haversine length algorithm](https://github.com/georust/geo/pull/183)

## 0.7.0

* [Add `Line` to the `Geometry` `enum`](https://github.com/georust/geo/pull/179)
* [Use new bulk-load method for initial R* Tree population](https://github.com/georust/geo/pull/178)
* [Add PostGIS and GeoJSON integration/conversions](https://github.com/georust/geo/pull/180)

## 0.6.3

* [Initial implementation of a `ClosestPoint` algorithm](https://github.com/georust/geo/pull/167)

## 0.6.2

* [Add a prelude: `use geo::prelude::*`](https://github.com/georust/geo/pull/162)

## 0.6.1

* [Add a `lines` iterator method on `LineString`](https://github.com/georust/geo/pull/160)
* [Implement `Contains<Polygon>` for `Polygon`](https://github.com/georust/geo/pull/159)
* [Correctly check for LineString containment in Polygon](https://github.com/georust/geo/pull/158)

## 0.6.0

* [Remove unnecessary trait bound on `Translate`](https://github.com/georust/geo/pull/148)
* [Topology preserving Visvalingam-Whyatt algorithm](https://github.com/georust/geo/pull/143)
* [Implement `Copy` for `Line`](https://github.com/georust/geo/pull/150)
* [Rewrite `RotatePoint` impls to be generic](https://github.com/georust/geo/pull/153)
* [Add associated return type for `BoundingBox`](https://github.com/georust/geo/pull/156)
* [Add associated return type for `Centroid`](https://github.com/georust/geo/pull/154)

## 0.5.0

* [Reimplement `Translate` trait using `MapCoords`](https://github.com/georust/geo/pull/145)

## 0.4.13

* [Implement Simplification traits for more types](https://github.com/georust/geo/pull/135)
* [Add a MapCoords trait](https://github.com/georust/geo/pull/136)

## 0.4.12

* [Improve robustness when calculating distance from a point to a
line-segment](https://github.com/georust/geo/pull/139)

## 0.4.11

* [Add `From`, `IntoIterator`, `Into` impls; add doc comments](https://github.com/georust/geo/pull/131)

## 0.4.10

* [Add `Translation` trait.](https://github.com/georust/geo/pull/128)

## 0.4.9

* [Add `Into` trait implementations.](https://github.com/georust/geo/pull/129)

## 0.4.8

* [Add `HaversineDestination` algorithm trait](https://github.com/georust/geo/pull/124)

## 0.4.7

* [Serializing/deserializing via serde](https://github.com/georust/geo/pull/125)

## 0.4.6

* [Fix incorrect usage of `abs_sub`](https://github.com/georust/geo/pull/120)

## 0.4.5

* [Add `Line` type (representing a line segment)](https://github.com/georust/geo/pull/118)

## 0.4.4

* [Quickhull orientation fix](https://github.com/georust/geo/pull/110)
* [Implement distance traits for more geometries](https://github.com/georust/geo/pull/113)
* [Correctly calculate centroid for complex polygons](https://github.com/georust/geo/pull/112)
* [Add `Orient` trait for polygon](https://github.com/georust/geo/pull/108)
* [Add geometry rotation](https://github.com/georust/geo/pull/107)
* [Add extreme point-finding](https://github.com/georust/geo/pull/114)
* [Add contains point impl for bbox](https://github.com/georust/geo/commit/3e00ef94c3d69e6d0b1caab86224469ced9444e6)

## 0.4.3

* [Implement Point to multipart geometry distance methods](https://github.com/georust/geo/pull/104)
* [Fixture cleanup](https://github.com/georust/geo/pull/105)

## 0.4.2

* [Fix Haversine distance implementation bug](https://github.com/georust/geo/pull/101)

## 0.4.1

* [Implement convex hull algorithm](https://github.com/georust/geo/pull/89)

## 0.4.0

* [Implement Haversine algorithm](https://github.com/georust/geo/pull/90)
* [fix when multipolygon composed of two polygons of opposite clockwise](https://github.com/georust/geo/commits/master)
* [Migrate from 'num' to 'num_traits' crate](https://github.com/georust/geo/pull/86)

## 0.3.2

* [Add Visvalingam-Whyatt line-simplification algorithm](https://github.com/georust/geo/pull/84)

## 0.3.1

* [Within Epsilon matcher](https://github.com/georust/geo/pull/82)

## 0.3.0

* [Add named fields for the `Polygon` structure](https://github.com/georust/geo/pull/68)

## 0.2.8

* [Implement `Intersects<Bbox<T>> for Polygon`](https://github.com/georust/geo/pull/76)

## 0.2.7

* [Implement `Intersects<Polygon<T>> for Polygon`](https://github.com/georust/geo/issues/69)

## 0.2.6

* [Add Point to Polygon and Point to LineString distance methods](https://github.com/georust/geo/pull/61)

## 0.2.5

* [Implement LineString simplification](https://github.com/georust/geo/pull/55)

## 0.2.4

* [Performance improvements when iterating over pairs of coordinates](https://github.com/georust/geo/pull/50)

## 0.2.3

* [Add type Bbox and trait BoundingBox](https://github.com/georust/geo/pull/41)

## 0.2.2

* [Add the Length trait and implement Length for LineString and MultiLineString](https://github.com/georust/geo/pull/44)

## 0.2.1

* [Modify area for Polygon to consider also the isles](https://github.com/georust/geo/pull/43)
* [Add area trait to MultiPolygon](https://github.com/georust/geo/pull/43)

## 0.2.0

* [Data structures and traits are now generic (previously all were `f64`)](https://github.com/georust/geo/pull/30)
* [`geo::COORD_PRECISION` is now `f32` (previously was `f64`)](https://github.com/georust/geo/pull/40)

## 0.1.1

* [`Intersects` trait bugfixes](https://github.com/georust/geo/pull/34)

## 0.1.0

* [Add `Area` trait](https://github.com/georust/geo/pull/31)
* [Add `Contains` trait](https://github.com/georust/geo/pull/31)
* [Add `Distance` trait, remove `Point::distance_to`](https://github.com/georust/geo/pull/31)
* [Add `Intersects` trait](https://github.com/georust/geo/pull/31)
* [Implement `Centroid` trait for `MultiPolygon`](https://github.com/georust/geo/pull/31)

## 0.0.7

* [Implement `Centroid` trait, `Point::distance_to` method](https://github.com/georust/geo/pull/24)
