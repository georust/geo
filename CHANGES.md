# Changes

## geo 0.9.0 (unreleased)

* Make serde an optional dependency for `geo`, rename feature to `use-serde`
  * <https://github.com/georust/rust-geo/pull/209>
* Use the `proj` crate, rename feature to `use-proj`
  * <https://github.com/georust/rust-geo/pull/214>
* Return unboxed iterators from `LineString::lines`, `Winding::points_cw`, and `Winding::points_ccw`
  * <https://github.com/georust/rust-geo/pull/218>
* Fix compilation errors when using the `proj` feature
  * <https://github.com/georust/rust-geo/commit/0924f3179c95bfffb847562ee91675d7aa8454f5>
* Add `Polygon`-`Polygon` and `LineString`-`LineString` distance

## geo 0.8.3

* Reexport core types from `geo-types`
  * <https://github.com/georust/rust-geo/pull/201>

## geo-types 0.1.0

* New crate with core types from `geo`
  * <https://github.com/georust/rust-geo/pull/201>

## geo 0.8.2

* Fix documentation generation on docs.rs
  * https://github.com/georust/rust-geo/pull/202

## geo 0.8.1

* Fix centroid calculation for degenerate polygons
  * <https://github.com/georust/rust-geo/pull/203>

## geo 0.8.0

* Prefix Euclidean distance/length traits with 'Euclidean'.
  * <https://github.com/georust/rust-geo/pull/200>
* Bump num-traits: 0.1 → 0.2
  * <https://github.com/georust/rust-geo/pull/188>
* Implement `SpatialObject` for `Line` type
  * <https://github.com/georust/rust-geo/pull/181>
* Implement a `TryMapCoords` trait
  * <https://github.com/georust/rust-geo/pull/191>
  * <https://github.com/georust/rust-geo/pull/197>
* Impl Polygon convexity function on the type
  * <https://github.com/georust/rust-geo/pull/195>
* Implement rust-proj as an optional feature within geo
  * <https://github.com/georust/rust-geo/pull/192>

## geo 0.7.4

* [`cross_prod` method added to `Point`](https://github.com/georust/rust-geo/pull/189)

## geo 0.7.3

* [Allow coordinates to be more types (not just `Float`s)](https://github.com/georust/rust-geo/pull/187)

## geo 0.7.2

* [Easy methods to convert a Geometry to the underlying type](https://github.com/georust/rust-geo/pull/184)
* [Map coords inplace](https://github.com/georust/rust-geo/pull/170)
* [Added bearing trait]https://github.com/georust/rust-geo/pull/186)
* [Winding/Orientation for LineStrings](https://github.com/georust/rust-geo/pull/169)

## geo 0.7.1

* [Add Haversine length algorithm](https://github.com/georust/rust-geo/pull/183)

## geo 0.7.0

* [Add `Line` to the `Geometry` `enum`](https://github.com/georust/rust-geo/pull/179)
* [Use new bulk-load method for initial R* Tree population](https://github.com/georust/rust-geo/pull/178)
* [Add PostGIS and GeoJSON integration/conversions](https://github.com/georust/rust-geo/pull/180)

## geo 0.6.3

* [Initial implementation of a `ClosestPoint` algorithm](https://github.com/georust/rust-geo/pull/167)

## geo 0.6.2

* [Add a prelude: `use geo::prelude::*`](https://github.com/georust/rust-geo/pull/162)

## geo 0.6.1

* [Add a `lines` iterator method on `LineString`](https://github.com/georust/rust-geo/pull/160)
* [Implement `Contains<Polygon>` for `Polygon`](https://github.com/georust/rust-geo/pull/159)
* [Correctly check for LineString containment in Polygon](https://github.com/georust/rust-geo/pull/158)

## geo 0.6.0

* [Remove unnecessary trait bound on `Translate`](https://github.com/georust/rust-geo/pull/148)
* [Topology preserving Visvalingam-Whyatt algorithm](https://github.com/georust/rust-geo/pull/143)
* [Implement `Copy` for `Line`](https://github.com/georust/rust-geo/pull/150)
* [Rewrite `RotatePoint` impls to be generic](https://github.com/georust/rust-geo/pull/153)
* [Add associated return type for `BoundingBox`](https://github.com/georust/rust-geo/pull/156)
* [Add associated return type for `Centroid`](https://github.com/georust/rust-geo/pull/154)

## geo 0.5.0

* [Reimplement `Translate` trait using `MapCoords`](https://github.com/georust/rust-geo/pull/145)

## geo 0.4.13

* [Implement Simplification traits for more types](https://github.com/georust/rust-geo/pull/135)
* [Add a MapCoords trait](https://github.com/georust/rust-geo/pull/136)

## geo 0.4.12

* [Improve robustness when calculating distance from a point to a
line-segment](https://github.com/georust/rust-geo/pull/139)

## geo 0.4.11

* [Add `From`, `IntoIterator`, `Into` impls; add doc comments](https://github.com/georust/rust-geo/pull/131)

## geo 0.4.10

* [Add `Translation` trait.](https://github.com/georust/rust-geo/pull/128)

## geo 0.4.9

* [Add `Into` trait implementations.](https://github.com/georust/rust-geo/pull/129)

## geo 0.4.8

* [Add `HaversineDestination` algorithm trait](https://github.com/georust/rust-geo/pull/124)

## geo 0.4.7

* [Serializing/deserializing via serde](https://github.com/georust/rust-geo/pull/125)

## geo 0.4.6

* [Fix incorrect usage of `abs_sub`](https://github.com/georust/rust-geo/pull/120)

## geo 0.4.5

* [Add `Line` type (representing a line segment)](https://github.com/georust/rust-geo/pull/118)

## geo 0.4.4

* [Quickhull orientation fix](https://github.com/georust/rust-geo/pull/110)
* [Implement distance traits for more geometries](https://github.com/georust/rust-geo/pull/113)
* [Correctly calculate centroid for complex polygons](https://github.com/georust/rust-geo/pull/112)
* [Add `Orient` trait for polygon](https://github.com/georust/rust-geo/pull/108)
* [Add geometry rotation](https://github.com/georust/rust-geo/pull/107)
* [Add extreme point-finding](https://github.com/georust/rust-geo/pull/114)
* [Add contains point impl for bbox](https://github.com/georust/rust-geo/commit/3e00ef94c3d69e6d0b1caab86224469ced9444e6)

## geo 0.4.3

* [Implement Point to multipart geometry distance methods](https://github.com/georust/rust-geo/pull/104)
* [Fixture cleanup](https://github.com/georust/rust-geo/pull/105)

## geo 0.4.2

* [Fix Haversine distance implementation bug](https://github.com/georust/rust-geo/pull/101)

## geo 0.4.1

* [Implement convex hull algorithm](https://github.com/georust/rust-geo/pull/89)

## geo 0.4.0

* [Implement Haversine algorithm](https://github.com/georust/rust-geo/pull/90)
* [fix when multipolygon composed of two polygons of opposite clockwise](https://github.com/georust/rust-geo/commits/master)
* [Migrate from 'num' to 'num_traits' crate](https://github.com/georust/rust-geo/pull/86)

## geo 0.3.2

* [Add Visvalingam-Whyatt line-simplification algorithm](https://github.com/georust/rust-geo/pull/84)

## geo 0.3.1

* [Within Epsilon matcher](https://github.com/georust/rust-geo/pull/82)

## geo 0.3.0

* [Add named fields for the `Polygon` structure](https://github.com/georust/rust-geo/pull/68)

## geo 0.2.8

* [Implement `Intersects<Bbox<T>> for Polygon`](https://github.com/georust/rust-geo/pull/76)

## geo 0.2.7

* [Implement `Intersects<Polygon<T>> for Polygon`](https://github.com/georust/rust-geo/issues/69)

## geo 0.2.6

* [Add Point to Polygon and Point to LineString distance methods](https://github.com/georust/rust-geo/pull/61)

## geo 0.2.5

* [Implement LineString simplification](https://github.com/georust/rust-geo/pull/55)

## geo 0.2.4

* [Performance improvements when iterating over pairs of coordinates](https://github.com/georust/rust-geo/pull/50)

## geo 0.2.3

* [Add type Bbox and trait BoundingBox](https://github.com/georust/rust-geo/pull/41)

## geo 0.2.2

* [Add the Length trait and implement Length for LineString and MultiLineString](https://github.com/georust/rust-geo/pull/44)

## geo 0.2.1

* [Modify area for Polygon to consider also the isles](https://github.com/georust/rust-geo/pull/43)
* [Add area trait to MultiPolygon](https://github.com/georust/rust-geo/pull/43)

## geo 0.2.0

* [Data structures and traits are now generic (previously all were `f64`)](https://github.com/georust/rust-geo/pull/30)
* [`geo::COORD_PRECISION` is now `f32` (previously was `f64`)](https://github.com/georust/rust-geo/pull/40)

## geo 0.1.1

* [`Intersects` trait bugfixes](https://github.com/georust/rust-geo/pull/34)

## geo 0.1.0

* [Add `Area` trait](https://github.com/georust/rust-geo/pull/31)
* [Add `Contains` trait](https://github.com/georust/rust-geo/pull/31)
* [Add `Distance` trait, remove `Point::distance_to`](https://github.com/georust/rust-geo/pull/31)
* [Add `Intersects` trait](https://github.com/georust/rust-geo/pull/31)
* [Implement `Centroid` trait for `MultiPolygon`](https://github.com/georust/rust-geo/pull/31)

## geo 0.0.7

* [Implement `Centroid` trait, `Point::distance_to` method](https://github.com/georust/rust-geo/pull/24)
