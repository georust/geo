use std::{fmt::Display, str::FromStr};

use approx::RelativeEq;
use geo_types::Polygon;
use num_traits::Signed;
use wkt::{ToWkt, TryFromWkt};

use crate::{
    area::twice_signed_ring_area, coordinate_position::CoordPos, dimensions::Dimensions,
    monotone::monotone_subdivision, GeoFloat, GeoNum, Relate,
};

pub(super) fn init_log() {
    use pretty_env_logger::env_logger;
    use std::io::Write;
    let _ = env_logger::builder()
        .format(|buf, record| writeln!(buf, "[{}] - {}", record.level(), record.args()))
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();
}

fn twice_polygon_area<T: GeoNum + Signed>(poly: &Polygon<T>) -> T {
    let mut area = twice_signed_ring_area(poly.exterior()).abs();
    for interior in poly.interiors() {
        area = area - twice_signed_ring_area(interior).abs();
    }
    area
}

fn check_monotone_subdivision<T: GeoFloat + FromStr + Default + Display + RelativeEq>(wkt: &str) {
    init_log();
    eprintln!("input: {wkt}");
    let input = Polygon::<T>::try_from_wkt_str(wkt).unwrap();
    let area = twice_polygon_area(&input);
    let subdivisions = monotone_subdivision([input.clone()]);
    eprintln!("Got {} subdivisions", subdivisions.len());

    let mut sub_area = T::zero();
    for (i, d1) in subdivisions.iter().enumerate() {
        for (j, d2) in subdivisions.iter().enumerate() {
            if i >= j {
                continue;
            }
            let p1 = d1.clone().into_polygon();
            let p2 = d2.clone().into_polygon();
            let im = p1.relate(&p2);
            let intin = im.get(CoordPos::Inside, CoordPos::Inside);
            assert!(intin == Dimensions::Empty);
        }
    }
    for div in subdivisions {
        let (mut top, bot) = div.into_ls_pair();
        top.0.extend(bot.0.into_iter().rev().skip(1));
        if !top.is_closed() {
            // This branch is for debugging
            // It will never be reached unless assertions elsewhere are commented.
            error!("Got an unclosed line string");
            error!("{}", top.to_wkt());
        } else {
            let poly = Polygon::new(top, vec![]);
            sub_area = sub_area + twice_polygon_area(&poly);
            info!("{}", poly.to_wkt());

            let im = poly.relate(&input);
            assert!(im.is_within());
        }
    }
    assert_relative_eq!(area, sub_area);
}

#[test]
fn test_monotone_subdivision_simple() {
    let input = "POLYGON((0 0,5 5,3 0,5 -5,0 0))";
    check_monotone_subdivision::<f64>(input);
}

#[test]
fn test_monotone_subdivision_merge_split() {
    let input = "POLYGON((-5 -5, -3 0, -5 5, 5 5,3 0,5 -5))";
    check_monotone_subdivision::<f64>(input);
}

#[test]
fn test_complex() {
    let input = "POLYGON ((140 300, 140 100, 140 70, 340 220, 187 235, 191 285, 140 300), 
        (140 100, 150 100, 150 110, 140 100))";
    check_monotone_subdivision::<f64>(input);
}

#[test]
fn test_complex2() {
    let input = "POLYGON ((100 100, 200 150, 100 200, 200 250, 100 300, 400 300,
       300 200, 400 100, 100 100))";
    check_monotone_subdivision::<f64>(input);
}

#[test]
fn test_complex3() {
    let input = "POLYGON((0 0,11.9 1,5.1 2,6.6 3,13.3 4,
        20.4 5,11.5 6,1.3 7,19.4 8,15.4 9,2.8 10,7.0 11,
        13.7 12,24.0 13,2.6 14,9.6 15,0.2 16,250 16,
        67.1 15,66.1 14,61.2 13,76.4 12,75.1 11,88.3 10,
        75.3 9,63.8 8,84.2 7,77.5 6,95.9 5,83.8 4,
        86.9 3,64.5 2,68.3 1,99.6 0,0 0))";
    check_monotone_subdivision::<f64>(input);
}

#[test]
fn test_tangent() {
    let input = "POLYGON ((60 60, 60 200, 240 200, 240 60, 60 60), 
    (60 140, 110 170, 110 100, 80 100, 60 140))";
    check_monotone_subdivision::<f64>(input);
}
