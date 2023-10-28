use super::*;
use geo_types::*;
use wkt::TryFromWkt;

// helper

fn load_wkt(data_str: &str) -> Result<Vec<Polygon<f32>>, String> {
    let GeometryCollection(data) =
        GeometryCollection::<f32>::try_from_wkt_str(data_str).map_err(|e| format!("{e:?}"))?;
    let data = data
        .into_iter()
        .filter_map(|g| g.try_into().ok())
        .collect::<Vec<_>>();
    Ok(data)
}

#[test]
fn basic_intersection_compiles() {
    let zero = Coord::zero();
    let one = Coord { x: 1.0, y: 1.0 };
    let rect1 = Rect::new(zero, one * 2.0);
    let rect2 = Rect::new(one, one * 3.0);

    SpadeBoolops::intersection(&rect1.to_polygon(), &rect2.to_polygon()).unwrap();
}

#[test]
fn load_star_works() {
    _ = pretty_env_logger::try_init();
    let data = include_str!("./data/star.wkt");
    let data = load_wkt(data).unwrap();
    info!("{data:?}");
}
