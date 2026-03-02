use console_log::init_with_level;
use geo::MultiPoint;
use geo::{KMeans, Point};
use log::{info, Level};
use rand::Rng;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
/// Mininal kmeans app.
pub fn compute_clusters() -> JsValue {
    let _ = init_with_level(Level::Debug);
    let mut rng = rand::rng();

    let points = (0..75)
        .map(|_| {
            Point::new(
                rng.random_range(5..=195) as f32,
                rng.random_range(5..=195) as f32,
            )
        })
        .collect::<Vec<Point<f32>>>();

    let mp = MultiPoint::new(points);

    info!("Starting computation");
    let labels = mp.kmeans(2).unwrap();
    info!("Computation complete");

    info!("{labels:?}");
    // Two clusters should be found
    let cluster_0 = labels
        .iter()
        .zip(mp.0.iter())
        .filter(|(index, _)| **index == 0)
        .map(|(_, p)| [p.x(), p.y()])
        .collect::<Vec<_>>();
    let cluster_1 = labels
        .iter()
        .zip(mp.0.iter())
        .filter(|(index, _)| **index == 1)
        .map(|(_, p)| [p.x(), p.y()])
        .collect::<Vec<_>>();

    info!("{cluster_0:?}");
    info!("{cluster_1:?}");

    serde_wasm_bindgen::to_value(&vec![cluster_0, cluster_1]).unwrap()
}
