use geo_types::polygon;
use wkt::ToWkt;

use crate::monotone::monotone_subdivision;

#[test]
fn test_monotone_subdivision_simple() {
    let input = polygon!(
        exterior: [
            (x: 0, y: 0),
            (x: 5, y: 5),
            (x: 3, y: 0),
            (x: 5, y: -5),
        ],
        interiors: [],
    );
    eprintln!("input: {}", input.to_wkt());

    let subdivisions = monotone_subdivision(input);
    eprintln!("Got {} subdivisions", subdivisions.len());
    for div in subdivisions {
        eprintln!("subdivision: {:?}", div);
    }
}
