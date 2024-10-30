# Changes

## Unreleased

- BREAKING: Rename `CoordTrait::nth_unchecked` -> `CoordTrait::nth_or_panic` since it is not `unsafe`.
  - <https://github.com/georust/geo/pull/1242>

## 0.1.1

- Fix `TriangleTrait::second` and `TriangleTrait::third` to return the second and third coordinates instead of the first.
  - <https://github.com/georust/geo/pull/1236>

## 0.1.0

- Initial release
