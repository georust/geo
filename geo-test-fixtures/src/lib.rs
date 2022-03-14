use std::{fs, iter::FromIterator, path::PathBuf, str::FromStr};

use geo_types::LineString;
use wkt::{Geometry, Wkt, WktFloat};

pub fn louisiana<T>() -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    line_string("louisiana.wkt")
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

fn line_string<T>(name: &str) -> LineString<T>
where
    T: WktFloat + Default + FromStr,
{
    let wkt = Wkt::from_str(&file(name)).unwrap();
    match wkt.item {
        Geometry::LineString(line_string) => {
            LineString::from_iter(line_string.0.into_iter().map(|coord| (coord.x, coord.y)))
        }
        _ => unreachable!(),
    }
}

fn file(name: &str) -> String {
    let mut res = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    res.push("fixtures");
    res.push(name);
    fs::read_to_string(res).unwrap()
}
