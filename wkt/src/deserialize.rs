//! This module provides deserialisation to WKT primitives using [`serde`]

use crate::{Geometry, Wkt, WktFloat};
use serde::de::{Deserializer, Error, Visitor};
use std::{
    default::Default,
    fmt::{self, Debug},
    marker::PhantomData,
    str::FromStr,
};

struct WktVisitor<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for WktVisitor<T> {
    fn default() -> Self {
        WktVisitor {
            _marker: PhantomData::default(),
        }
    }
}

impl<'de, T> Visitor<'de> for WktVisitor<T>
where
    T: FromStr + Default + Debug + WktFloat,
{
    type Value = Wkt<T>;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "a valid WKT format")
    }
    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Wkt::from_str(s).map_err(|e| serde::de::Error::custom(e))
    }
}

impl<'de, T> serde::Deserialize<'de> for Wkt<T>
where
    T: FromStr + Default + Debug + WktFloat,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(WktVisitor::default())
    }
}

struct GeometryVisitor<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for GeometryVisitor<T> {
    fn default() -> Self {
        GeometryVisitor {
            _marker: PhantomData::default(),
        }
    }
}

impl<'de, T> Visitor<'de> for GeometryVisitor<T>
where
    T: FromStr + Default + WktFloat,
{
    type Value = Geometry<T>;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "a valid WKT format")
    }
    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let wkt = Wkt::from_str(s).map_err(|e| serde::de::Error::custom(e))?;
        Ok(wkt.item)
    }
}

impl<'de, T> serde::Deserialize<'de> for Geometry<T>
where
    T: FromStr + Default + WktFloat,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(GeometryVisitor::default())
    }
}

/// Deserializes directly from WKT format into a [`geo_types::Geometry`].
/// ```
/// # extern crate wkt;
/// # extern crate geo_types;
/// # extern crate serde_json;
/// use geo_types::Geometry;
///
/// #[derive(serde::Deserialize)]
/// struct MyType {
///     #[serde(deserialize_with = "wkt::deserialize_geometry")]
///     pub geometry: Geometry<f64>,
/// }
///
/// let json = r#"{ "geometry": "POINT (3.14 42)" }"#;
/// let my_type: MyType = serde_json::from_str(json).unwrap();
/// assert!(matches!(my_type.geometry, Geometry::Point(_)));
/// ```
#[cfg(feature = "geo-types")]
pub fn deserialize_geometry<'de, D, T>(deserializer: D) -> Result<geo_types::Geometry<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + Default + WktFloat,
{
    use serde::Deserialize;
    Geometry::deserialize(deserializer)
        .and_then(|g: Geometry<T>| g.try_into().map_err(D::Error::custom))
}

/// Deserializes directly from WKT format into an `Option<geo_types::Point>`.
///
/// # Examples
///
///
/// ```
/// # extern crate wkt;
/// # extern crate geo_types;
/// # extern crate serde_json;
/// use geo_types::Point;
///
/// #[derive(serde::Deserialize)]
/// struct MyType {
///     #[serde(deserialize_with = "wkt::deserialize_point")]
///     pub geometry: Option<Point<f64>>,
/// }
///
/// let json = r#"{ "geometry": "POINT (3.14 42)" }"#;
/// let my_type: MyType = serde_json::from_str(json).unwrap();
/// assert!(matches!(my_type.geometry, Some(Point(_))));
///
/// let json = r#"{ "geometry": "POINT EMPTY" }"#;
/// let my_type: MyType = serde_json::from_str(json).unwrap();
/// assert!(matches!(my_type.geometry, None));
/// ```
#[cfg(feature = "geo-types")]
pub fn deserialize_point<'de, D, T>(
    deserializer: D,
) -> Result<Option<geo_types::Point<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + Default + WktFloat,
{
    use serde::Deserialize;
    Wkt::deserialize(deserializer).and_then(|wkt: Wkt<T>| {
        geo_types::Geometry::try_from(wkt)
            .map_err(D::Error::custom)
            .and_then(|geom| {
                use geo_types::Geometry::*;
                match geom {
                    Point(p) => Ok(Some(p)),
                    MultiPoint(mp) if mp.0.is_empty() => Ok(None),
                    _ => geo_types::Point::try_from(geom)
                        .map(Some)
                        .map_err(D::Error::custom),
                }
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        types::{Coord, Point},
        Geometry,
    };
    use serde::de::{
        value::{Error, StrDeserializer},
        Deserializer, Error as _, IntoDeserializer,
    };

    mod wkt {
        use super::*;

        #[test]
        fn deserialize() {
            let deserializer: StrDeserializer<'_, Error> = "POINT (10 20.1)".into_deserializer();
            let wkt = deserializer
                .deserialize_any(WktVisitor::<f64>::default())
                .unwrap();
            assert!(matches!(
                wkt.item,
                Geometry::Point(Point(Some(Coord {
                    x: _, // floating-point types cannot be used in patterns
                    y: _, // floating-point types cannot be used in patterns
                    z: None,
                    m: None,
                })))
            ));
        }

        #[test]
        fn deserialize_error() {
            let deserializer: StrDeserializer<'_, Error> = "POINT (10 20.1A)".into_deserializer();
            let wkt = deserializer.deserialize_any(WktVisitor::<f64>::default());
            assert_eq!(
                wkt.unwrap_err(),
                Error::custom("Expected a number for the Y coordinate")
            );
        }
    }

    mod geometry {
        use super::*;

        #[test]
        fn deserialize() {
            let deserializer: StrDeserializer<'_, Error> = "POINT (42 3.14)".into_deserializer();
            let geometry = deserializer
                .deserialize_any(GeometryVisitor::<f64>::default())
                .unwrap();
            assert!(matches!(
                geometry,
                Geometry::Point(Point(Some(Coord {
                    x: _, // floating-point types cannot be used in patterns
                    y: _, // floating-point types cannot be used in patterns
                    z: None,
                    m: None,
                })))
            ));
        }

        #[test]
        fn deserialize_error() {
            let deserializer: StrDeserializer<'_, Error> = "POINT (42 PI3.14)".into_deserializer();
            let geometry = deserializer.deserialize_any(GeometryVisitor::<f64>::default());
            assert_eq!(
                geometry.unwrap_err(),
                Error::custom("Expected a number for the Y coordinate")
            );
        }
    }
}
