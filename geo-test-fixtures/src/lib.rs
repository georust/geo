use std::{fs, iter::FromIterator, path::PathBuf, str::FromStr};

use geo_types::{LineString, MultiPolygon, Point, Polygon};
use wkt::{Geometry, Wkt, WktFloat};

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
pub fn nl_plots<T>() -> MultiPolygon<T>
where
    T: WktFloat + Default + FromStr,
{
    multi_polygon("nl_plots.wkt")
}

fn line_string<T>(name: &str) -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    let wkt = Wkt::from_str(&file(name)).unwrap();
    match wkt.item {
        Geometry::LineString(line_string) => wkt_line_string_to_geo(&line_string),
        _ => unreachable!(),
    }
}

fn polygon<T>(name: &str) -> Polygon<T>
where
    T: WktFloat + Default + FromStr,
{
    let wkt = Wkt::from_str(&file(name)).unwrap();
    match wkt.item {
        Geometry::Polygon(wkt_polygon) => wkt_polygon_to_geo(&wkt_polygon),
        _ => unreachable!(),
    }
}

fn multi_polygon<T>(name: &str) -> MultiPolygon<T>
where
    T: WktFloat + Default + FromStr,
{
    let wkt = Wkt::from_str(&file(name)).unwrap();
    match wkt.item {
        Geometry::MultiPolygon(multi_polygon) => wkt_multi_polygon_to_geo(&multi_polygon),
        _ => unreachable!(),
    }
}

fn wkt_line_string_to_geo<T>(line_string: &wkt::types::LineString<T>) -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    LineString::from_iter(line_string.0.iter().map(|coord| (coord.x, coord.y)))
}

fn wkt_polygon_to_geo<T>(polygon: &wkt::types::Polygon<T>) -> Polygon<T>
where
    T: WktFloat + Default + FromStr,
{
    let exterior: LineString<T> = wkt_line_string_to_geo(&polygon.0[0]);
    let interiors: Vec<LineString<T>> = polygon.0[1..].iter().map(wkt_line_string_to_geo).collect();

    Polygon::new(exterior, interiors)
}

fn wkt_multi_polygon_to_geo<T>(multi_polygon: &wkt::types::MultiPolygon<T>) -> MultiPolygon<T>
where
    T: WktFloat + Default + FromStr,
{
    let polygons: Vec<Polygon<T>> = multi_polygon.0.iter().map(wkt_polygon_to_geo).collect();
    MultiPolygon(polygons)
}

fn file(name: &str) -> String {
    let mut res = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    res.push("fixtures");
    res.push(name);
    fs::read_to_string(res).unwrap()
}
