use geo::{line_string, Centroid};

fn main() {
    let linestring = geo::line_string![
        (x: 40.02f64, y: 116.34),
        (x: 41.02f64, y: 116.34),
    ];
    println!("Centroid {:?}", linestring.centroid());
}
