use console_log::init_with_level;
use geo::KMeans;
use geo::{point, MultiPoint};
use log::{info, Level};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
/// Mininal kmeans app.
pub fn compute_clusters() -> JsValue {
    let _ = init_with_level(Level::Debug);

    let points = MultiPoint::new(vec![
        point!(x: 25.0_f64, y: 25.0_f64),
        point!(x: 35.0_f64, y: 25.0_f64),
        point!(x: 25.0_f64, y: 35.0_f64),
        point!(x: 100.0, y: 100.0_f64),
        point!(x: 110.0_f64, y: 100.0_f64),
        point!(x: 100.0_f64, y: 110.0_f64),
    ]);

    info!("Starting computation");
    let labels = points.kmeans(2).unwrap();
    info!("Computation complete");

    let _ = info!("{labels:?}");
    // Two clusters should be found
    let cluster_0 = labels
        .iter()
        .zip(points.0.iter())
        .filter(|(index, _)| **index == 0)
        .map(|(i, p)| [p.x(), p.y()])
        .collect::<Vec<_>>();
    let cluster_1 = labels
        .iter()
        .zip(points.0.iter())
        .filter(|(index, _)| **index == 1)
        .map(|(i, p)| [p.x(), p.y()])
        .collect::<Vec<_>>();

    let _ = info!("{cluster_0:?}");
    let _ = info!("{cluster_1:?}");

    serde_wasm_bindgen::to_value(&vec![cluster_0, cluster_1]).unwrap()
}
