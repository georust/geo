use std::{fmt::Display, str::FromStr};

use geo_types::Polygon;
use num_traits::Signed;
use wkt::{ToWkt, TryFromWkt};

use crate::{area::twice_signed_ring_area, monotone::monotone_subdivision, GeoNum};

pub(super) fn init_log() {
    use pretty_env_logger::env_logger;
    use std::io::Write;
    let _ = env_logger::builder()
        .format(|buf, record| writeln!(buf, "[{}] - {}", record.level(), record.args()))
        // .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();
}

fn twice_polygon_area<T: GeoNum + Signed>(poly: &Polygon<T>) -> T {
    let mut area = twice_signed_ring_area(&poly.exterior()).abs();
    for interior in poly.interiors() {
        area = area - twice_signed_ring_area(interior).abs();
    }
    area
}

fn check_monotone_subdivision<T: GeoNum + Signed + Display + FromStr + Default>(wkt: &str) {
    init_log();
    eprintln!("input: {wkt}");
    let input = Polygon::<T>::try_from_wkt_str(wkt).unwrap();
    let area = twice_polygon_area(&input);
    let subdivisions = monotone_subdivision(input);
    eprintln!("Got {} subdivisions", subdivisions.len());

    let mut sub_area = T::zero();
    for div in subdivisions {
        sub_area = sub_area + twice_polygon_area(&div.clone().into_polygon());
        let (top, bot) = div.into_ls_pair();
        eprintln!("top: {}", top.to_wkt());
        eprintln!("bot: {}", bot.to_wkt());
    }

    assert_eq!(area, sub_area);
}

#[test]
fn test_monotone_subdivision_simple() {
    let input = "POLYGON((0 0,5 5,3 0,5 -5,0 0))";
    check_monotone_subdivision::<i64>(&input);
}

#[test]
fn test_monotone_subdivision_merge_split() {
    let input = "POLYGON((-5 -5, -3 0, -5 5, 5 5,3 0,5 -5))";
    check_monotone_subdivision::<i64>(&input);
}

#[test]
fn test_complex() {
    let input = "POLYGON ((140 300, 140 100, 140 70, 340 220, 187 235, 191 285, 140 300), 
        (140 100, 150 100, 150 110, 140 100))";
    check_monotone_subdivision::<i64>(&input);
}
