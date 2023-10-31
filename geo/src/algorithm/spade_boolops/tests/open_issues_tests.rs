use crate::SpadeBoolops;
use geo_types::{Coord, LineString, MultiPolygon, Polygon};

#[test]
fn no_1103_union_for_f32_polys() {
    let polygons = [
        Polygon::<f32>::new(
            LineString::from(vec![
                Coord {
                    x: 3.349_365_2,
                    y: -55.801_27,
                },
                Coord {
                    x: 171.224_43,
                    y: -300.0,
                },
                Coord {
                    x: 291.841_64,
                    y: -300.0,
                },
                Coord {
                    x: 46.650_635,
                    y: -30.801_27,
                },
                Coord {
                    x: 3.349_365_2,
                    y: -55.801_27,
                },
            ]),
            Vec::new(),
        ),
        Polygon::<f32>::new(
            LineString::from(vec![
                Coord {
                    x: 46.650_635,
                    y: -30.801_27,
                },
                Coord {
                    x: 291.841_64,
                    y: -300.0,
                },
                Coord {
                    x: 300.0,
                    y: -228.340_03,
                },
                Coord {
                    x: -3.349_365_2,
                    y: 55.801_27,
                },
                Coord {
                    x: 46.650_635,
                    y: -30.801_27,
                },
            ]),
            Vec::new(),
        ),
        Polygon::<f32>::new(
            LineString::from(vec![
                Coord {
                    x: -46.650_635,
                    y: 30.801_27,
                },
                Coord {
                    x: 195.837_28,
                    y: -300.0,
                },
                Coord {
                    x: 300.0,
                    y: -228.340_03,
                },
                Coord {
                    x: -3.349_365_2,
                    y: 55.801_27,
                },
                Coord {
                    x: -46.650_635,
                    y: 30.801_27,
                },
            ]),
            Vec::new(),
        ),
        Polygon::<f32>::new(
            LineString::from(vec![
                Coord {
                    x: 3.349_365_2,
                    y: -55.801_27,
                },
                Coord {
                    x: 171.224_43,
                    y: -300.0,
                },
                Coord {
                    x: 195.837_28,
                    y: -300.0,
                },
                Coord {
                    x: -46.650_635,
                    y: 30.801_27,
                },
                Coord {
                    x: 3.349_365_2,
                    y: -55.801_27,
                },
            ]),
            Vec::new(),
        ),
    ];

    let mut multi = MultiPolygon::new(Vec::new());
    for poly in polygons {
        multi = MultiPolygon::union(&multi, &MultiPolygon::new(vec![poly])).unwrap();
    }
}

#[test]
fn no_1103_union_for_f64_polys() {
    let polygons = [
        Polygon::<f64>::new(
            LineString::from(vec![
                Coord {
                    x: 3.349365234375,
                    y: -55.80126953125,
                },
                Coord {
                    x: 171.224_426_269_531_25,
                    y: -300.0,
                },
                Coord {
                    x: 291.841_644_287_109_4,
                    y: -300.0,
                },
                Coord {
                    x: 46.650_634_765_625,
                    y: -30.801_269_531_25,
                },
                Coord {
                    x: 3.349_365_234_375,
                    y: -55.801_269_531_25,
                },
            ]),
            Vec::new(),
        ),
        Polygon::<f64>::new(
            LineString::from(vec![
                Coord {
                    x: 46.650_634_765_625,
                    y: -30.801_269_531_25,
                },
                Coord {
                    x: 291.841_644_287_109_4,
                    y: -300.0,
                },
                Coord {
                    x: 300.0,
                    y: -228.340_026_855_468_75,
                },
                Coord {
                    x: -3.349_365_234_375,
                    y: 55.801_269_531_25,
                },
                Coord {
                    x: 46.650_634_765_625,
                    y: -30.801_269_531_25,
                },
            ]),
            Vec::new(),
        ),
        Polygon::<f64>::new(
            LineString::from(vec![
                Coord {
                    x: -46.650_634_765_625,
                    y: 30.801_269_531_25,
                },
                Coord {
                    x: 195.837_280_273_437_5,
                    y: -300.0,
                },
                Coord {
                    x: 300.0,
                    y: -228.340_026_855_468_75,
                },
                Coord {
                    x: -3.349_365_234_375,
                    y: 55.801_269_531_25,
                },
                Coord {
                    x: -46.650_634_765_625,
                    y: 30.801_269_531_25,
                },
            ]),
            Vec::new(),
        ),
        Polygon::<f64>::new(
            LineString::from(vec![
                Coord {
                    x: 3.349_365_234_375,
                    y: -55.801_269_531_25,
                },
                Coord {
                    x: 171.224_426_269_531_25,
                    y: -300.0,
                },
                Coord {
                    x: 195.837_280_273_437_5,
                    y: -300.0,
                },
                Coord {
                    x: -46.650_634_765_625,
                    y: 30.801_269_531_25,
                },
                Coord {
                    x: 3.349_365_234_375,
                    y: -55.801_269_531_25,
                },
            ]),
            Vec::new(),
        ),
    ];

    let mut multi = MultiPolygon::new(Vec::new());
    for poly in polygons {
        multi = MultiPolygon::union(&multi, &MultiPolygon::new(vec![poly])).unwrap();
    }
}
