//! Conversion between [`geo-types`] and [`postgis`] types.
//!
//! # Examples
//!
//! Convert a `postgis` point to a `geo-types` point:
//!
//! [`geo-types`]: https://crates.io/crates/geo-types
//! [`postgis`]: https://crates.io/crates/postgis
//!
//! ```rust
//! use geo_postgis::FromPostgis;
//!
//! let postgis_point = postgis::ewkb::Point { x: 1., y: -2., srid: None };
//!
//! let geo_point = geo_types::Point::from_postgis(&postgis_point);
//!
//! assert_eq!(
//!     geo_types::point!(x: 1., y: -2.),
//!     geo_point,
//! );
//! ```
//!
//! Convert a `geo-types` point to a `postgis` point:
//!
//! ```rust
//! use geo_postgis::ToPostgis;
//!
//! let geo_point = geo_types::point!(x: 1., y: -2.);
//!
//! let postgis_point = geo_point.to_postgis_with_srid(None);
//!
//! assert_eq!(
//!     postgis::ewkb::Point { x: 1., y: -2., srid: None },
//!     postgis_point,
//! );
//! ```

mod to_postgis;
pub use to_postgis::ToPostgis;

mod from_postgis;
pub use from_postgis::FromPostgis;
