# Changes

## 0.4.6

* [Fix incorrect usage of `abs_sub`](https://github.com/georust/rust-geo/pull/120)

## 0.4.5

* [Add `Line` type (representing a line segment)](https://github.com/georust/rust-geo/pull/118)

## 0.4.4

* [Quickhull orientation fix](https://github.com/georust/rust-geo/pull/110)
* [Implement distance traits for more geometries](https://github.com/georust/rust-geo/pull/113)
* [Correctly calculate centroid for complex polygons](https://github.com/georust/rust-geo/pull/112)
* [Add `Orient` trait for polygon](https://github.com/georust/rust-geo/pull/108)
* [Add geometry rotation](https://github.com/georust/rust-geo/pull/107)
* [Add extreme point-finding](https://github.com/georust/rust-geo/pull/114)
* [Add contains point impl for bbox](https://github.com/georust/rust-geo/commit/3e00ef94c3d69e6d0b1caab86224469ced9444e6)

## 0.4.3

* [Implement Point to multipart geometry distance methods](https://github.com/georust/rust-geo/pull/104)
* [Fixture cleanup](https://github.com/georust/rust-geo/pull/105)

## 0.4.2

* [Fix Haversine distance implementation bug](https://github.com/georust/rust-geo/pull/101)

## 0.4.1

* [Implement convex hull algorithm](https://github.com/georust/rust-geo/pull/89)

## 0.4.0

* [Implement Haversine algorithm](https://github.com/georust/rust-geo/pull/90)
* [fix when multipolygon composed of two polygons of opposite clockwise](https://github.com/georust/rust-geo/commits/master)
* [Migrate from 'num' to 'num_traits' crate](https://github.com/georust/rust-geo/pull/86)

## 0.3.2

* [Add Visvalingam-Whyatt line-simplification algorithm](https://github.com/georust/rust-geo/pull/84)

## 0.3.1

* [Within Epsilon matcher](https://github.com/georust/rust-geo/pull/82)

## 0.3.0

* [Add named fields for the `Polygon` structure](https://github.com/georust/rust-geo/pull/68)

## 0.2.8

* [Implement `Intersects<Bbox<T>> for Polygon`](https://github.com/georust/rust-geo/pull/76)

## 0.2.7

* [Implement `Intersects<Polygon<T>> for Polygon`](https://github.com/georust/rust-geo/issues/69)

## 0.2.6

* [Add Point to Polygon and Point to LineString distance methods](https://github.com/georust/rust-geo/pull/61)

## 0.2.5

* [Implement LineString simplification](https://github.com/georust/rust-geo/pull/55)

## 0.2.4

* [Performance improvements when iterating over pairs of coordinates](https://github.com/georust/rust-geo/pull/50)

## 0.2.3

* [Add type Bbox and trait BoundingBox](https://github.com/georust/rust-geo/pull/41)

## 0.2.2

* [Add the Length trait and implement Length for LineString and MultiLineString](https://github.com/georust/rust-geo/pull/44)

## 0.2.1

* [Modify area for Polygon to consider also the isles](https://github.com/georust/rust-geo/pull/43)
* [Add area trait to MultiPolygon](https://github.com/georust/rust-geo/pull/43)

## 0.2.0

* [Data structures and traits are now generic (previously all were `f64`)](https://github.com/georust/rust-geo/pull/30)
* [`geo::COORD_PRECISION` is now `f32` (previously was `f64`)](https://github.com/georust/rust-geo/pull/40)

## 0.1.1

* [`Intersects` trait bugfixes](https://github.com/georust/rust-geo/pull/34)

## 0.1.0

* [Add `Area` trait](https://github.com/georust/rust-geo/pull/31)
* [Add `Contains` trait](https://github.com/georust/rust-geo/pull/31)
* [Add `Distance` trait, remove `Point::distance_to`](https://github.com/georust/rust-geo/pull/31)
* [Add `Intersects` trait](https://github.com/georust/rust-geo/pull/31)
* [Implement `Centroid` trait for `MultiPolygon`](https://github.com/georust/rust-geo/pull/31)

## 0.0.7

* [Implement `Centroid` trait, `Point::distance_to` method](https://github.com/georust/rust-geo/pull/24)
