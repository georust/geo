[![geo](https://avatars1.githubusercontent.com/u/10320338?v=4&s=50)](https://github.com/georust)

[![geo on Crates.io](https://img.shields.io/crates/v/geo.svg?color=brightgreen)](https://crates.io/crates/geo)
[![Coverage Status](https://img.shields.io/coverallsCoverage/github/georust/geo.svg)](https://coveralls.io/github/georust/geo?branch=trying)
[![Documentation](https://img.shields.io/docsrs/geo/latest.svg)](https://docs.rs/geo)
[![Discord](https://img.shields.io/discord/598002550221963289)](https://discord.gg/Fp2aape)

# geo

## Geospatial Primitives, Algorithms, and Utilities

### Chat or ask questions on [Discord](https://discord.gg/Fp2aape)

The `geo` crate provides geospatial primitive types such as `Point`, `LineString`, and `Polygon`, and provides algorithms and operations such as:
- Full DE-9IM support and topological relationship calculations such as containment and intersection
- Affine operations on geometries (scale, rotate, skew, translate)
- Boolean operations on geometries (clip, union, difference, intersection, xor)
- Buffer / offset operations on geometries
- Clustering operations such as DBSCAN and _k_-means
- Euclidean, as well as spherical, haversine and other non-planar length and distance calculations
- Support for projecting and converting between coordinate reference systems using PROJ
- IO using the `geojson` and `geozero` crates.

Please refer to [the documentation](https://docs.rs/geo) for a complete list.

The primitive types also provide the basis for other functionality in the `Geo` ecosystem, including:

- [Coordinate transformation and projection](https://github.com/georust/proj)
- Serialization to and from [GeoJSON](https://github.com/georust/geojson) and [WKT](https://github.com/georust/wkt)
- [Geocoding](https://github.com/georust/geocoding)
- [Working with GPS data](https://github.com/georust/gpx)

## Example

```rust
// primitives
use geo::{line_string, polygon};

// algorithms
use geo::ConvexHull;

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

## Web Applications

When targetting for the browser (ie target="wasm*-unknown-unknown"), selecting a source of randomness is troublesome. This is how to include the geo library for the web.

```toml
geo = { path ="0.32", features =  [ "earcutr", "spade", "multithreading"], default-features = false }
```

The rand crate is no longer a dependant and the functionality associated the kmeans module is removed.

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
