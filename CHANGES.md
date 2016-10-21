# Changes

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
