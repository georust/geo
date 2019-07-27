# Changes

## geo 0.12.2

* Introduce `point!`, `line_string!`, and `polygon!` macros.
  * <https://github.com/georust/geo/pull/352>

## geo-types 0.4.3

* Introduce `point!`, `line_string!`, and `polygon!` macros.
  * <https://github.com/georust/geo/pull/352>
  * <https://github.com/georust/geo/pull/357>
* Add `Rect` constructor that enforces `min.{x,y} < max.{x,y}`
  * <https://github.com/georust/geo/pull/360>

## geo 0.12.1

* Add `FrechetDistance` algorithm
  * <https://github.com/georust/geo/pull/348>

## geo-types 0.4.2

* Add `Polygon::num_coords`
  * <https://github.com/georust/geo/pull/348>

## geo-types 0.4.1

* Add `Polygon::interiors_push` - Adds an interior ring to a `Polygon`
  * <https://github.com/georust/geo/pull/347>

## geo 0.12.0

* Bump `geo-types` dependency to 0.4.0
* Bump `rstar` and `proj` dependencies
  * <https://github.com/georust/geo/pull/346>
* Implement `Centroid` for `MultiPoint`
  * <https://github.com/georust/geo/pull/322>

## geo-types 0.4.0

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

## geo 0.11.0

* Replace the [spade](https://crates.io/crates/spade) crate with the [rstar](https://crates.io/crates/rstar) crate
  * <https://github.com/georust/geo/pull/314>
* Remove unnecessary algorithm trait bounds
  * <https://github.com/georust/geo/pull/320/>

## geo-types 0.3.0

* Replace the [spade](https://crates.io/crates/spade) crate with the [rstar](https://crates.io/crates/rstar) crate
  * <https://github.com/georust/geo/pull/314>
* Remove unnecessary algorithm trait bounds
  * <https://github.com/georust/geo/pull/320/>

## geo 0.10.3

* Add `MapCoords` for `Rect`s
  * <https://github.com/georust/geo/commit/11e4b67ae5fa658bd556eea96ba6fd49f32921c4>
* Rewrite vincenty/haversine docs; specify param/return units.
  * <https://github.com/georust/geo/commit/6ca45c347c53c5f0fd41b90ff5d0ba67d1b2ec15>
* `Area` can work on some non-`Float` geometries (e.g. `Rect<Integer>`)
  * <https://github.com/georust/geo/commit/1efd87a9bf3f4140f252014b59ff174af8e014aa>

## geo-types 0.2.2

* Fix misnamed `serde` feature flag.
  * <https://github.com/georust/geo/pull/316>
* Add `width` and `height` helpers on `Rect`.
  * <https://github.com/georust/geo/pull/317>

## geo-types 0.2.1

* Add `to_lines` method on a `Triangle`
  * <https://github.com/georust/geo/pull/313>

## geo 0.10.2

* Add `to_degrees` and `to_radians` methods on `Point`s
  * <https://github.com/georust/geo/pull/306>

## geo 0.10.1

* Fix some edge case on centroid computation
  * <https://github.com/georust/geo/pull/305>

## geo-types 0.2.0

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

## geo 0.10.0

* Remove unnecessary borrows in function params for `Copy` types.
  * <https://github.com/georust/geo/pull/265>
* Rename bounding ‘box’ to ‘rect’; move structure to geo-types.
  * <https://github.com/georust/geo/pull/295>

## geo 0.9.1

* Fix Line-Polygon euclidean distance
  * <https://github.com/georust/geo/pull/226>
* Implement `EuclideanDistance` for `MultiPolygon` to `Line` and `Line` to `MultiPolygon`
  * <https://github.com/georust/geo/pull/227>
* Add `Line`-`LineString` euclidean distance
  * <https://github.com/georust/geo/pull/232>
* Add `VincentyDistance` and `VincentyLength` algorithms
  * <https://github.com/georust/geo/pull/213>
* Add `HaversineIntermediate` algorithm

## geo 0.9.0

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

## geo-types 0.1.1

* Allow LineString creation from vec of two-element CoordinateType array
  * <https://github.com/georust/geo/pull/223>

## geo 0.8.3

* Reexport core types from `geo-types`
  * <https://github.com/georust/geo/pull/201>

## geo-types 0.1.0

* New crate with core types from `geo`
  * <https://github.com/georust/geo/pull/201>

## geo 0.8.2

* Fix documentation generation on docs.rs
  * <https://github.com/georust/geo/pull/202>

## geo 0.8.1

* Fix centroid calculation for degenerate polygons
  * <https://github.com/georust/geo/pull/203>

## geo 0.8.0

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

## geo 0.7.4

* [`cross_prod` method added to `Point`](https://github.com/georust/geo/pull/189)

## geo 0.7.3

* [Allow coordinates to be more types (not just `Float`s)](https://github.com/georust/geo/pull/187)

## geo 0.7.2

* [Easy methods to convert a Geometry to the underlying type](https://github.com/georust/geo/pull/184)
* [Map coords inplace](https://github.com/georust/geo/pull/170)
* [Added bearing trait]https://github.com/georust/geo/pull/186)
* [Winding/Orientation for LineStrings](https://github.com/georust/geo/pull/169)

## geo 0.7.1

* [Add Haversine length algorithm](https://github.com/georust/geo/pull/183)

## geo 0.7.0

* [Add `Line` to the `Geometry` `enum`](https://github.com/georust/geo/pull/179)
* [Use new bulk-load method for initial R* Tree population](https://github.com/georust/geo/pull/178)
* [Add PostGIS and GeoJSON integration/conversions](https://github.com/georust/geo/pull/180)

## geo 0.6.3

* [Initial implementation of a `ClosestPoint` algorithm](https://github.com/georust/geo/pull/167)

## geo 0.6.2

* [Add a prelude: `use geo::prelude::*`](https://github.com/georust/geo/pull/162)

## geo 0.6.1

* [Add a `lines` iterator method on `LineString`](https://github.com/georust/geo/pull/160)
* [Implement `Contains<Polygon>` for `Polygon`](https://github.com/georust/geo/pull/159)
* [Correctly check for LineString containment in Polygon](https://github.com/georust/geo/pull/158)

## geo 0.6.0

* [Remove unnecessary trait bound on `Translate`](https://github.com/georust/geo/pull/148)
* [Topology preserving Visvalingam-Whyatt algorithm](https://github.com/georust/geo/pull/143)
* [Implement `Copy` for `Line`](https://github.com/georust/geo/pull/150)
* [Rewrite `RotatePoint` impls to be generic](https://github.com/georust/geo/pull/153)
* [Add associated return type for `BoundingBox`](https://github.com/georust/geo/pull/156)
* [Add associated return type for `Centroid`](https://github.com/georust/geo/pull/154)

## geo 0.5.0

* [Reimplement `Translate` trait using `MapCoords`](https://github.com/georust/geo/pull/145)

## geo 0.4.13

* [Implement Simplification traits for more types](https://github.com/georust/geo/pull/135)
* [Add a MapCoords trait](https://github.com/georust/geo/pull/136)

## geo 0.4.12

* [Improve robustness when calculating distance from a point to a
line-segment](https://github.com/georust/geo/pull/139)

## geo 0.4.11

* [Add `From`, `IntoIterator`, `Into` impls; add doc comments](https://github.com/georust/geo/pull/131)

## geo 0.4.10

* [Add `Translation` trait.](https://github.com/georust/geo/pull/128)

## geo 0.4.9

* [Add `Into` trait implementations.](https://github.com/georust/geo/pull/129)

## geo 0.4.8

* [Add `HaversineDestination` algorithm trait](https://github.com/georust/geo/pull/124)

## geo 0.4.7

* [Serializing/deserializing via serde](https://github.com/georust/geo/pull/125)

## geo 0.4.6

* [Fix incorrect usage of `abs_sub`](https://github.com/georust/geo/pull/120)

## geo 0.4.5

* [Add `Line` type (representing a line segment)](https://github.com/georust/geo/pull/118)

## geo 0.4.4

* [Quickhull orientation fix](https://github.com/georust/geo/pull/110)
* [Implement distance traits for more geometries](https://github.com/georust/geo/pull/113)
* [Correctly calculate centroid for complex polygons](https://github.com/georust/geo/pull/112)
* [Add `Orient` trait for polygon](https://github.com/georust/geo/pull/108)
* [Add geometry rotation](https://github.com/georust/geo/pull/107)
* [Add extreme point-finding](https://github.com/georust/geo/pull/114)
* [Add contains point impl for bbox](https://github.com/georust/geo/commit/3e00ef94c3d69e6d0b1caab86224469ced9444e6)

## geo 0.4.3

* [Implement Point to multipart geometry distance methods](https://github.com/georust/geo/pull/104)
* [Fixture cleanup](https://github.com/georust/geo/pull/105)

## geo 0.4.2

* [Fix Haversine distance implementation bug](https://github.com/georust/geo/pull/101)

## geo 0.4.1

* [Implement convex hull algorithm](https://github.com/georust/geo/pull/89)

## geo 0.4.0

* [Implement Haversine algorithm](https://github.com/georust/geo/pull/90)
* [fix when multipolygon composed of two polygons of opposite clockwise](https://github.com/georust/geo/commits/master)
* [Migrate from 'num' to 'num_traits' crate](https://github.com/georust/geo/pull/86)

## geo 0.3.2

* [Add Visvalingam-Whyatt line-simplification algorithm](https://github.com/georust/geo/pull/84)

## geo 0.3.1

* [Within Epsilon matcher](https://github.com/georust/geo/pull/82)

## geo 0.3.0

* [Add named fields for the `Polygon` structure](https://github.com/georust/geo/pull/68)

## geo 0.2.8

* [Implement `Intersects<Bbox<T>> for Polygon`](https://github.com/georust/geo/pull/76)

## geo 0.2.7

* [Implement `Intersects<Polygon<T>> for Polygon`](https://github.com/georust/geo/issues/69)

## geo 0.2.6

* [Add Point to Polygon and Point to LineString distance methods](https://github.com/georust/geo/pull/61)

## geo 0.2.5

* [Implement LineString simplification](https://github.com/georust/geo/pull/55)

## geo 0.2.4

* [Performance improvements when iterating over pairs of coordinates](https://github.com/georust/geo/pull/50)

## geo 0.2.3

* [Add type Bbox and trait BoundingBox](https://github.com/georust/geo/pull/41)

## geo 0.2.2

* [Add the Length trait and implement Length for LineString and MultiLineString](https://github.com/georust/geo/pull/44)

## geo 0.2.1

* [Modify area for Polygon to consider also the isles](https://github.com/georust/geo/pull/43)
* [Add area trait to MultiPolygon](https://github.com/georust/geo/pull/43)

## geo 0.2.0

* [Data structures and traits are now generic (previously all were `f64`)](https://github.com/georust/geo/pull/30)
* [`geo::COORD_PRECISION` is now `f32` (previously was `f64`)](https://github.com/georust/geo/pull/40)

## geo 0.1.1

* [`Intersects` trait bugfixes](https://github.com/georust/geo/pull/34)

## geo 0.1.0

* [Add `Area` trait](https://github.com/georust/geo/pull/31)
* [Add `Contains` trait](https://github.com/georust/geo/pull/31)
* [Add `Distance` trait, remove `Point::distance_to`](https://github.com/georust/geo/pull/31)
* [Add `Intersects` trait](https://github.com/georust/geo/pull/31)
* [Implement `Centroid` trait for `MultiPolygon`](https://github.com/georust/geo/pull/31)

## geo 0.0.7

* [Implement `Centroid` trait, `Point::distance_to` method](https://github.com/georust/geo/pull/24)
