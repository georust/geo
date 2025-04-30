use geo_traits::to_geo::ToGeoGeometry;
use geo_traits_ext::{
    GeometryTraitExt, GeometryTypeExt, LineStringTraitExt, PointTraitExt, PolygonTraitExt,
};
use geo_types::{line_string, Geometry};
use geos::WKBWriter;

use crate::wkb::reader::read_wkb;

use super::data::*;

#[test]
fn read_point_srid() {
    let orig = point_2d();
    let mut geos_geom: geos::Geometry = (&orig).try_into().unwrap();
    geos_geom.set_srid(1);

    let mut wkb_writer = WKBWriter::new().unwrap();
    wkb_writer.set_include_SRID(true);
    let buf: Vec<u8> = wkb_writer.write_wkb(&geos_geom).unwrap().into();

    let retour = read_wkb(&buf).unwrap();
    assert_eq!(Geometry::Point(orig), retour.to_geometry());
}

#[test]
fn read_line_string_srid() {
    let orig = linestring_2d();

    let mut geos_geom: geos::Geometry = (&orig).try_into().unwrap();
    geos_geom.set_srid(1);

    let mut wkb_writer = WKBWriter::new().unwrap();
    wkb_writer.set_include_SRID(true);
    let buf: Vec<u8> = wkb_writer.write_wkb(&geos_geom).unwrap().into();

    let retour = read_wkb(&buf).unwrap();
    assert_eq!(Geometry::LineString(orig.clone()), retour.to_geometry());
}

#[test]
fn read_polygon() {
    let orig = polygon_2d();

    let mut geos_geom: geos::Geometry = (&orig).try_into().unwrap();
    geos_geom.set_srid(1);

    let mut wkb_writer = WKBWriter::new().unwrap();
    wkb_writer.set_include_SRID(true);
    let buf: Vec<u8> = wkb_writer.write_wkb(&geos_geom).unwrap().into();

    let retour = read_wkb(&buf).unwrap();
    assert_eq!(Geometry::Polygon(orig.clone()), retour.to_geometry());
}

#[test]
fn read_polygon_with_interior() {
    let orig = polygon_2d_with_interior();

    let mut geos_geom: geos::Geometry = (&orig).try_into().unwrap();
    geos_geom.set_srid(1);

    let mut wkb_writer = WKBWriter::new().unwrap();
    wkb_writer.set_include_SRID(true);
    let buf: Vec<u8> = wkb_writer.write_wkb(&geos_geom).unwrap().into();

    let retour = read_wkb(&buf).unwrap();
    assert_eq!(Geometry::Polygon(orig.clone()), retour.to_geometry());
}

#[test]
fn read_multi_point() {
    let orig = multi_point_2d();

    let mut geos_geom: geos::Geometry = (&orig).try_into().unwrap();
    geos_geom.set_srid(1);

    let mut wkb_writer = WKBWriter::new().unwrap();
    wkb_writer.set_include_SRID(true);
    let buf: Vec<u8> = wkb_writer.write_wkb(&geos_geom).unwrap().into();

    let retour = read_wkb(&buf).unwrap();
    assert_eq!(Geometry::MultiPoint(orig.clone()), retour.to_geometry());
}

#[test]
fn read_multi_line_string() {
    let orig = multi_line_string_2d();

    let mut geos_geom: geos::Geometry = (&orig).try_into().unwrap();
    geos_geom.set_srid(1);

    let mut wkb_writer = WKBWriter::new().unwrap();
    wkb_writer.set_include_SRID(true);
    let buf: Vec<u8> = wkb_writer.write_wkb(&geos_geom).unwrap().into();

    let retour = read_wkb(&buf).unwrap();
    assert_eq!(
        Geometry::MultiLineString(orig.clone()),
        retour.to_geometry()
    );
}

#[test]
fn read_multi_polygon() {
    let orig = multi_polygon_2d();

    let mut geos_geom: geos::Geometry = (&orig).try_into().unwrap();
    geos_geom.set_srid(1);

    let mut wkb_writer = WKBWriter::new().unwrap();
    wkb_writer.set_include_SRID(true);
    let buf: Vec<u8> = wkb_writer.write_wkb(&geos_geom).unwrap().into();

    let retour = read_wkb(&buf).unwrap();
    assert_eq!(Geometry::MultiPolygon(orig.clone()), retour.to_geometry());
}

fn geometry_to_geos(geom: &geo_types::Geometry) -> geos::Geometry {
    match geom {
        Geometry::Point(inner) => geos::Geometry::try_from(inner).unwrap(),
        Geometry::MultiPoint(inner) => geos::Geometry::try_from(inner).unwrap(),
        Geometry::LineString(inner) => geos::Geometry::try_from(inner).unwrap(),
        Geometry::MultiLineString(inner) => geos::Geometry::try_from(inner).unwrap(),
        Geometry::Polygon(inner) => geos::Geometry::try_from(inner).unwrap(),
        Geometry::MultiPolygon(inner) => geos::Geometry::try_from(inner).unwrap(),
        Geometry::GeometryCollection(inner) => geometry_collection_to_geos(inner),
        _ => unimplemented!(),
    }
}

fn geometry_collection_to_geos(geom: &geo_types::GeometryCollection) -> geos::Geometry {
    let geoms: Vec<_> = geom.0.iter().map(geometry_to_geos).collect::<Vec<_>>();

    geos::Geometry::create_geometry_collection(geoms).unwrap()
}

#[test]
fn read_geometry_collection() {
    let orig = geometry_collection_2d();

    let mut geos_geom = geometry_collection_to_geos(&orig);
    geos_geom.set_srid(1);

    let mut wkb_writer = WKBWriter::new().unwrap();
    wkb_writer.set_include_SRID(true);
    let buf: Vec<u8> = wkb_writer.write_wkb(&geos_geom).unwrap().into();

    let retour = read_wkb(&buf).unwrap();
    assert_eq!(
        Geometry::GeometryCollection(orig.clone()),
        retour.to_geometry()
    );
}

#[test]
fn read_point_geo_coord() {
    let orig = point_2d();
    let geos_geom: geos::Geometry = (&orig).try_into().unwrap();
    let mut wkb_writer = WKBWriter::new().unwrap();
    let byte_orders = [geos::ByteOrder::LittleEndian, geos::ByteOrder::BigEndian];
    for byte_order in byte_orders {
        wkb_writer.set_wkb_byte_order(byte_order);
        let buf: Vec<u8> = wkb_writer.write_wkb(&geos_geom).unwrap().into();
        let wkb = read_wkb(&buf).unwrap();
        if let GeometryTypeExt::Point(pt) = wkb.as_type_ext() {
            let coord = pt.geo_coord();
            assert_eq!(coord, Some(orig.0));
        } else {
            panic!("Expected a Point");
        }
    }
}

#[test]
fn line_string_iterator() {
    let orig = line_string![
        (x: 0., y: 1.),
        (x: 1., y: 2.),
        (x: 2., y: 3.),
        (x: 3., y: 4.),
        (x: 4., y: 5.),
    ];
    let geos_geom: geos::Geometry = (&orig).try_into().unwrap();
    let mut wkb_writer = WKBWriter::new().unwrap();
    let buf: Vec<u8> = wkb_writer.write_wkb(&geos_geom).unwrap().into();
    let wkb = read_wkb(&buf).unwrap();
    match wkb.as_type_ext() {
        GeometryTypeExt::LineString(ls) => {
            let lines = ls.lines().collect::<Vec<_>>();
            assert_eq!(lines.len(), 4);
            for (i, line) in lines.iter().enumerate() {
                assert_eq!(line.start, orig.0[i]);
                assert_eq!(line.end, orig.0[i + 1]);
            }

            let coords = ls.coord_iter().collect::<Vec<_>>();
            assert_eq!(coords.len(), 5);
            for (i, coord) in coords.iter().enumerate() {
                assert_eq!(coord, &orig.0[i]);
            }
        }
        _ => unreachable!(),
    }
}

#[test]
fn linear_ring_iterator() {
    let orig = polygon_2d();
    let geos_geom: geos::Geometry = (&orig).try_into().unwrap();
    let mut wkb_writer = WKBWriter::new().unwrap();
    let buf: Vec<u8> = wkb_writer.write_wkb(&geos_geom).unwrap().into();
    let wkb = read_wkb(&buf).unwrap();
    match wkb.as_type_ext() {
        GeometryTypeExt::Polygon(poly) => {
            let lr = poly.exterior_ext().unwrap();
            let lines = lr.lines().collect::<Vec<_>>();
            assert_eq!(lines.len(), 4);
            for (i, line) in lines.iter().enumerate() {
                assert_eq!(line.start, orig.exterior()[i]);
                assert_eq!(line.end, orig.exterior()[i + 1]);
            }

            let coords = lr.coord_iter().collect::<Vec<_>>();
            assert_eq!(coords.len(), 5);
            for (i, coord) in coords.iter().enumerate() {
                assert_eq!(coord, &orig.exterior()[i]);
            }
        }
        _ => unreachable!(),
    }
}
