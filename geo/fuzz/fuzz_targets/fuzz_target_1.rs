#![no_main]

use geo::algorithm::convex_hull::ConvexHull;
use libfuzzer_sys::fuzz_target;

fn geo_line_string_wkt(line_string: &geo::LineString<f32>) -> String {
    let mut wkt = String::from("LINESTRING (");

    for (i, coord) in line_string.0.iter().enumerate() {
        wkt.push_str(&format!("{} {}", coord.x, coord.y));
        if i != (line_string.0.len() - 1) {
            wkt.push_str(", ");
        }
    }

    wkt.push_str(")");

    wkt
}

fuzz_target!(|geo_line_string: geo::LineString<f32>| {
    //println!("{:?}", geo_line_string);

    let wkt = geo_line_string_wkt(&geo_line_string);
    //println!("{:?}", wkt);

    let gdal_line_string = gdal::vector::Geometry::from_wkt(&wkt).unwrap();
    let gdal_convex_hull = gdal_line_string.convex_hull().unwrap();

    //println!("{:?}", gdal_line_string.wkt().unwrap());
    //if geo_line_string.0.len() > 4 {
    //    return;
    //}

    //println!("-----");
    //for coord in &geo_line_string.0 {
    //    println!("Coordinate {{ x: {:.32}, y: {:.32} }},", coord.x, coord.y);
    //}

    let geo_convex_hull = geo_line_string.convex_hull();
});
