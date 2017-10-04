[![Build Status](https://travis-ci.org/georust/rust-geo.svg?branch=master)](https://travis-ci.org/georust/rust-geo)
[![geo on Crates.io](https://meritbadge.herokuapp.com/geo)](https://crates.io/crates/geo)

[![rust-geo](https://avatars1.githubusercontent.com/u/10320338?v=4&s=100)]()

# rust-geo
## Geospatial Primitives, Algorithms, and Utilities

The `geo` crate provides a number of geospatial primitive types such as `Point`, `LineString` and `Polyon`, and provides algorithms and operations such as:
- Area and centroid calculation
- Simplification and Convex Hull operations
- Euclidean and Haversine distance measurement
- Intersection checks
- Affine transforms such as rotation and translation

While this crate is pre-`1.0` and should be considered neither stable nor feature-complete, it is actively maintained and provides an extensive suite of types and algorithms for geospatial or more general 2D applications.

Please refer to [the documentation](https://docs.rs/geo) for a complete list.

## Example
```rust
use geo::{Polygon, LineString};
use geo::convexhull::ConvexHull;

let coords = vec![(1.0, 0.0), (2.0, 1.0), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
// conversions to geo types are provided from several kinds of coordinate sequences
let poly = Polygon::new(coords.into(), vec![]);
// uses the QuickHull algorithm to calculate the polygon's convex hull
let hull = poly.convex_hull();
let correct = vec![(4.0, 0.0), (4.0, 1.0), (1.0, 4.0), (0.0, 4.0), (0.0, 0.0), (4.0, 0.0)]
assert_eq!(hull.exterior, correct.into());
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
