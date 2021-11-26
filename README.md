[![geo](https://avatars1.githubusercontent.com/u/10320338?v=4&s=50)](https://github.com/georust)

[![Build Status](https://travis-ci.org/georust/geo.svg?branch=master)](https://travis-ci.org/georust/geo)
[![geo on Crates.io](https://img.shields.io/crates/v/geo.svg)](https://crates.io/crates/geo)
[![Coverage Status](https://coveralls.io/repos/github/georust/geo/badge.svg?branch=trying)](https://coveralls.io/github/georust/geo?branch=trying)
[![Documentation](https://docs.rs/geo/badge.svg)](https://docs.rs/geo)

# geo

## Geospatial Primitives, Algorithms, and Utilities

The `geo` crate provides geospatial primitive types such as `Point`, `LineString`, and `Polygon`, and provides algorithms and operations such as:
- Area and centroid calculation
- Simplification and convex hull operations
- Euclidean and Haversine distance measurement
- Intersection checks
- Affine transforms such as rotation and translation.

Please refer to [the documentation](https://docs.rs/geo) for a complete list.

The primitive types also provide the basis for other functionality in the `Geo` ecosystem, including:

- [Coordinate transformation and projection](https://github.com/georust/proj)
- Serialization to and from [GeoJSON](https://github.com/georust/geojson) and [WKT](https://github.com/georust/wkt)
- [Geocoding](https://github.com/georust/geocoding)
- [Working with GPS data](https://github.com/georust/gpx)

## Example

```rust
use geo::{line_string, polygon};
use geo::convex_hull::ConvexHull;

// An L shape
let poly = polygon![
    (x: 0.0, y: 0.0),
    (x: 4.0, y: 0.0),
    (x: 4.0, y: 1.0),
    (x: 1.0, y: 1.0),
    (x: 1.0, y: 4.0),
    (x: 0.0, y: 4.0),
    (x: 0.0, y: 0.0),
];

// Calculate the polygon's convex hull
let hull = poly.convex_hull();

assert_eq!(
    hull.exterior(),
    &line_string![
        (x: 4.0, y: 0.0),
        (x: 4.0, y: 1.0),
        (x: 1.0, y: 4.0),
        (x: 0.0, y: 4.0),
        (x: 0.0, y: 0.0),
        (x: 4.0, y: 0.0),
    ]
);
```

## Contributing

Contributions are welcome! Have a look at the [issues](https://github.com/georust/geo/issues), and open a pull request if you'd like to add an algorithm or some functionality.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
