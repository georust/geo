use geo::algorithm::concave_hull::ConcaveHull;
use geo::algorithm::convex_hull::ConvexHull;
use geo::coords_iter::CoordsIter;
use geo::{Coordinate, Point};
use geo_types::MultiPoint;
use std::fs::File;
use std::io::Write;

fn generate_polygon_str(coords: &[Coordinate<f64>]) -> String {
    let mut points_str = String::from("");
    for coord in coords {
        points_str.push_str(format!("{},{} ", coord.x(), coord.y()).as_ref());
    }
    return format!(
        "    <polygon points=\"{}\" fill=\"none\" stroke=\"black\"/>\n",
        points_str
    );
}

fn generate_consecutive_circles(coords: &[Coordinate<f64>]) -> String {
    let mut circles_str = String::from("");
    for coord in coords {
        circles_str.push_str(
            format!(
                "<circle cx=\"{}\" cy=\"{}\" r=\"1\"/>\n",
                coord.x(),
                coord.y()
            )
            .as_ref(),
        );
    }
    circles_str
}

fn produce_file_content(start_str: &str, mid_str: &str) -> String {
    let mut overall_string = start_str.to_string();
    overall_string.push_str(mid_str);
    overall_string.push_str("</svg>");
    overall_string
}

//Move the points such that they're clustered around the center of the image
fn move_points_in_viewbox(width: f64, height: f64, points: Vec<Point<f64>>) -> Vec<Point<f64>> {
    let mut new_points = vec![];
    for point in points {
        new_points.push(Point::new(
            point.x() + width / 2.0,
            point.y() + height / 2.0,
        ));
    }
    new_points
}

fn main() -> std::io::Result<()> {
    let mut points_file = File::create("points.svg")?;
    let mut concave_hull_file = File::create("concavehull.svg")?;
    let mut convex_hull_file = File::create("convexhull.svg")?;
    let width = 100;
    let height = 100;
    let svg_file_string = format!(
        "<svg viewBox=\"50 50 {} {}\" xmlns=\"http://www.w3.org/2000/svg\">\n",
        width, height
    );
    let norway = geo_test_fixtures::norway_main::<f64>();
    let v: Vec<_> = norway
        .into_iter()
        .map(|coord| Point::new(coord.x(), coord.y()))
        .collect();
    let moved_v = move_points_in_viewbox(width as f64, height as f64, v);
    let multipoint = MultiPoint::from(moved_v);
    let concave = multipoint.concave_hull(2.0);
    let convex = multipoint.convex_hull();
    let concave_polygon_str = generate_polygon_str(concave.exterior().inner());
    let convex_polygon_str = generate_polygon_str(convex.exterior().inner());
    let v_coords: Vec<_> = multipoint.coords_iter().collect();
    let circles_str = generate_consecutive_circles(&v_coords);
    let points_str = produce_file_content(&svg_file_string, &circles_str);
    let concave_hull_str = produce_file_content(&svg_file_string, &concave_polygon_str);
    let convex_hull_str = produce_file_content(&svg_file_string, &convex_polygon_str);

    points_file.write_all(points_str.as_ref())?;
    concave_hull_file.write_all(concave_hull_str.as_ref())?;
    convex_hull_file.write_all(convex_hull_str.as_ref())?;
    Ok(())
}
