# Changes

## Unreleased
### Changed
* Now accepts `MULTIPOINT`s with less parentheses, as output by `ST_AsText` in postgis:
  `MULTIPOINT(0 1, 2 3)` in addition to `MULTIPOINT((0 1), (2 3))`

## 0.9.2 - 2020-04-30
### Added
* Minimal support for JTS extension: `LINEARRING` by parsing it as a `LINESTRING`.
* Support `POINT EMPTY` in conversion to `geo_types`.
  Converts to `MultiPoint([])`.
  * <https://github.com/georust/wkt/pull/64>
### Fixed
* Some "numeric" characters like `¾` and `①` were being treated as digits.
### Changed
* Approximately 40% faster according to `cargo bench`.

## 0.9.1

* Add `serde::Deserialize` for `Wkt` and `Geometry`.
  * <https://github.com/georust/wkt/pull/59>
* Add helper functions for deserializing from WKT format into
  `geo_types::Geometry` and `geo_types::Point`
  * <https://github.com/georust/wkt/pull/59>
  * <https://github.com/georust/wkt/pull/62>

## 0.9.0

* WKT errors impl `std::error::Error`
  * <https://github.com/georust/wkt/pull/57>
* Add TryFrom for converting directly to geo-types::Geometry enum members, such
  as `geo_types::LineString::try_from(wkt)`
  * <https://github.com/georust/wkt/pull/57>
* Add `geo-types::Geometry::from(wkt)`
* BREAKING: update geo-types, apply new `geo_types::CoordFloat`
  * <https://github.com/georust/wkt/pull/53>
* BREAKING: Add Debug to Wkt structs by using new WktFloat instead of num_traits::Float
  * <https://github.com/georust/wkt/pull/54>

## 0.8.0

* update geo-types
