# Changes

## 0.3.0 - 2025-05-08

- BREAKING: All traits now extend `GeometryTrait`.
  - <https://github.com/georust/geo/pull/1346>
- Fix the lifetime annotation of `PointTrait` implemented for `geo_types::Point`.
  - <https://github.com/georust/geo/pull/1348>

## 0.2.0 - 2024-11-06

- BREAKING: Mark `CoordTrait::nth_unchecked` as `unsafe` and add `CoordTrait::nth_or_panic`.
  - <https://github.com/georust/geo/pull/1242>
- Make `geo-types` dependency optional for `geo-traits`.
  - <https://github.com/georust/geo/pull/1241>
- Add converter functions for `geo-traits` to `geo-types`.
  - <https://github.com/georust/geo/pull/1255>

## 0.1.1

- Fix `TriangleTrait::second` and `TriangleTrait::third` to return the second and third coordinates instead of the first.
  - <https://github.com/georust/geo/pull/1236>

## 0.1.0

- Initial release
