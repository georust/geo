use std::{path::PathBuf, str::FromStr};

use geo_types::{LineString, MultiPoint, MultiPolygon, Point, Polygon};
use wkt::{TryFromWkt, WktFloat};

pub mod checkerboard;

pub fn louisiana<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("louisiana.wkt")
}

pub fn baton_rouge<T>() -> Point<T>
where
    T: WktFloat + Default + FromStr,
{
    let x = T::from(-91.147385).unwrap();
    let y = T::from(30.471165).unwrap();
    Point::new(x, y)
}

pub fn east_baton_rouge<T>() -> Polygon<T>
where
    T: WktFloat + Default + FromStr,
{
    polygon("east_baton_rouge.wkt")
}

pub fn norway_main<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("norway_main.wkt")
}

pub fn norway_concave_hull<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("norway_concave_hull.wkt")
}

pub fn norway_convex_hull<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("norway_convex_hull.wkt")
}

pub fn norway_nonconvex_hull<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("norway_nonconvex_hull.wkt")
}

pub fn vw_orig<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("vw_orig.wkt")
}

pub fn vw_simplified<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("vw_simplified.wkt")
}

pub fn poly1<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("poly1.wkt")
}

pub fn poly1_hull<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("poly1_hull.wkt")
}

pub fn poly2<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("poly2.wkt")
}

pub fn poly2_hull<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("poly2_hull.wkt")
}

pub fn poly_in_ring<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("poly_in_ring.wkt")
}

pub fn ring<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("ring.wkt")
}

pub fn shell<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("shell.wkt")
}

// From https://geodata.nationaalgeoregister.nl/kadastralekaart/wfs/v4_0?request=GetFeature&service=WFS&srsName=EPSG:4326&typeName=kadastralekaartv4:perceel&version=2.0.0&outputFormat=json&bbox=165593,480993,166125,481552
pub fn nl_zones<T>() -> MultiPolygon<T>
where
    T: WktFloat + Default + FromStr,
{
    multi_polygon("nl_zones.wkt")
}

// From https://afnemers.ruimtelijkeplannen.nl/afnemers/services?request=GetFeature&service=WFS&srsName=EPSG:4326&typeName=Enkelbestemming&version=2.0.0&bbox=165618,480983,166149,481542";
pub fn nl_plots_wgs84<T>() -> MultiPolygon<T>
where
    T: WktFloat + Default + FromStr,
{
    multi_polygon("nl_plots.wkt")
}

pub fn nl_plots_epsg_28992<T>() -> MultiPolygon<T>
where
    T: WktFloat + Default + FromStr,
{
    // https://epsg.io/28992
    multi_polygon("nl_plots_epsg_28992.wkt")
}

fn line_string<T>(name: &str) -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    LineString::try_from_wkt_reader(file(name)).unwrap()
}

pub fn polygon<T>(name: &str) -> Polygon<T>
where
    T: WktFloat + Default + FromStr,
{
    Polygon::try_from_wkt_reader(file(name)).unwrap()
}

pub fn multi_polygon<T>(name: &str) -> MultiPolygon<T>
where
    T: WktFloat + Default + FromStr,
{
    MultiPolygon::try_from_wkt_reader(file(name)).unwrap()
}

pub fn multi_point<T>(name: &str) -> MultiPoint<T>
where
    T: WktFloat + Default + FromStr,
{
    MultiPoint::try_from_wkt_reader(file(name)).unwrap()
}

/// 104 UK cities used for Voronoi diagram testing
pub fn uk_cities<T>() -> MultiPoint<T>
where
    T: WktFloat + Default + FromStr,
{
    multi_point("voronoi/uk_cities.wkt")
}

/// 151 post box locations in Islington, London used for Voronoi diagram testing
pub fn islington_post_boxes<T>() -> MultiPoint<T>
where
    T: WktFloat + Default + FromStr,
{
    multi_point("voronoi/islington.wkt")
}

pub fn file(name: &str) -> std::fs::File {
    let mut res = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    res.push("fixtures");
    res.push(name);
    std::fs::File::open(&mut res).unwrap_or_else(|_| panic!("Can't open file: {res:?}"))
}
