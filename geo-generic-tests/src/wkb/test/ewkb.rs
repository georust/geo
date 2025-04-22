use geo_traits::to_geo::ToGeoGeometry;
use geo_types::Geometry;
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
