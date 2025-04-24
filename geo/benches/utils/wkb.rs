use geos::WKBWriter;

pub fn geo_to_wkb<G>(geo: G) -> Vec<u8>
where
    G: TryInto<geos::Geometry>,
{
    let geos_geom: geos::Geometry = match geo.try_into() {
        Ok(geos_geom) => geos_geom,
        Err(_) => panic!("Failed to convert to geos::Geometry"),
    };

    let mut wkb_writer = WKBWriter::new().unwrap();
    wkb_writer.write_wkb(&geos_geom).unwrap().into()
}
