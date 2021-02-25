# Changes

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
