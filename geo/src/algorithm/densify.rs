use crate::{
    CoordFloat, EuclideanLength, Line, LineInterpolatePoint, LineString, MultiLineString,
    MultiPolygon, Point, Polygon, Rect, Triangle,
};

/// Return a new linear geometry containing both existing and new interpolated coordinates with
/// a maximum distance of `max_distance` between them.
///
/// Note: `max_distance` must be greater than 0.
///
/// # Examples
/// ```
/// use geo::{coord, Line, LineString};
/// use geo::Densify;
///
/// let line: Line<f64> = Line::new(coord! {x: 0.0, y: 6.0}, coord! {x: 1.0, y: 8.0});
/// let correct: LineString<f64> = vec![[0.0, 6.0], [0.5, 7.0], [1.0, 8.0]].into();
/// let max_dist = 2.0;
/// let densified = line.densify(max_dist);
/// assert_eq!(densified, correct);
///```
pub trait Densify<F: CoordFloat> {
    type Output;

    fn densify(&self, max_distance: F) -> Self::Output;
}

// Helper for densification trait
fn densify_line<T: CoordFloat>(line: Line<T>, container: &mut Vec<Point<T>>, max_distance: T) {
    assert!(max_distance > T::zero());
    container.push(line.start_point());
    let num_segments = (line.euclidean_length() / max_distance)
        .ceil()
        .to_u64()
        .unwrap();
    // distance "unit" for this line segment
    let frac = T::one() / T::from(num_segments).unwrap();
    for segment_idx in 1..num_segments {
        let ratio = frac * T::from(segment_idx).unwrap();
        let interpolated_point = line
            .line_interpolate_point(ratio)
            .expect("ratio should be between 0..1");
        container.push(interpolated_point);
    }
}

impl<T> Densify<T> for MultiPolygon<T>
where
    T: CoordFloat,
    Line<T>: EuclideanLength<T>,
    LineString<T>: EuclideanLength<T>,
{
    type Output = MultiPolygon<T>;

    fn densify(&self, max_distance: T) -> Self::Output {
        MultiPolygon::new(
            self.iter()
                .map(|polygon| polygon.densify(max_distance))
                .collect(),
        )
    }
}

impl<T> Densify<T> for Polygon<T>
where
    T: CoordFloat,
    Line<T>: EuclideanLength<T>,
    LineString<T>: EuclideanLength<T>,
{
    type Output = Polygon<T>;

    fn densify(&self, max_distance: T) -> Self::Output {
        let densified_exterior = self.exterior().densify(max_distance);
        let densified_interiors = self
            .interiors()
            .iter()
            .map(|ring| ring.densify(max_distance))
            .collect();
        Polygon::new(densified_exterior, densified_interiors)
    }
}

impl<T> Densify<T> for MultiLineString<T>
where
    T: CoordFloat,
    Line<T>: EuclideanLength<T>,
    LineString<T>: EuclideanLength<T>,
{
    type Output = MultiLineString<T>;

    fn densify(&self, max_distance: T) -> Self::Output {
        MultiLineString::new(
            self.iter()
                .map(|linestring| linestring.densify(max_distance))
                .collect(),
        )
    }
}

impl<T> Densify<T> for LineString<T>
where
    T: CoordFloat,
    Line<T>: EuclideanLength<T>,
    LineString<T>: EuclideanLength<T>,
{
    type Output = LineString<T>;

    fn densify(&self, max_distance: T) -> Self::Output {
        if self.0.is_empty() {
            return LineString::new(vec![]);
        }

        let mut new_line = vec![];

        self.lines()
            .for_each(|line| densify_line(line, &mut new_line, max_distance));
        // we're done, push the last coordinate on to finish
        new_line.push(self.points().last().unwrap());
        LineString::from(new_line)
    }
}

impl<T> Densify<T> for Line<T>
where
    T: CoordFloat,
    Line<T>: EuclideanLength<T>,
    LineString<T>: EuclideanLength<T>,
{
    type Output = LineString<T>;

    fn densify(&self, max_distance: T) -> Self::Output {
        let mut new_line = vec![];
        densify_line(*self, &mut new_line, max_distance);
        // we're done, push the last coordinate on to finish
        new_line.push(self.end_point());
        LineString::from(new_line)
    }
}

impl<T> Densify<T> for Triangle<T>
where
    T: CoordFloat,
    Line<T>: EuclideanLength<T>,
    LineString<T>: EuclideanLength<T>,
{
    type Output = Polygon<T>;

    fn densify(&self, max_distance: T) -> Self::Output {
        self.to_polygon().densify(max_distance)
    }
}

impl<T> Densify<T> for Rect<T>
where
    T: CoordFloat,
    Line<T>: EuclideanLength<T>,
    LineString<T>: EuclideanLength<T>,
{
    type Output = Polygon<T>;

    fn densify(&self, max_distance: T) -> Self::Output {
        self.to_polygon().densify(max_distance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{coord, Coord};

    #[test]
    fn test_polygon_densify() {
        let linestring: LineString<f64> =
            vec![[-5.0, 0.0], [0.0, 5.0], [5.0, 0.0], [-5.0, 0.0]].into();
        let interior: LineString<f64> =
            vec![[-3.0, 0.0], [0.0, 3.0], [3.0, 0.0], [-3.0, 0.0]].into();
        let polygon = Polygon::new(linestring, vec![interior]);
        let correct_ext: LineString<f64> = LineString(vec![
            Coord { x: -5.0, y: 0.0 },
            Coord { x: -3.75, y: 1.25 },
            Coord { x: -2.5, y: 2.5 },
            Coord { x: -1.25, y: 3.75 },
            Coord { x: 0.0, y: 5.0 },
            Coord { x: 1.25, y: 3.75 },
            Coord { x: 2.5, y: 2.5 },
            Coord { x: 3.75, y: 1.25 },
            Coord { x: 5.0, y: 0.0 },
            Coord { x: 3.0, y: 0.0 },
            Coord { x: 1.0, y: 0.0 },
            Coord {
                x: -1.0000000000000009,
                y: 0.0,
            },
            Coord { x: -3.0, y: 0.0 },
            Coord { x: -5.0, y: 0.0 },
        ]);
        let correct_int: LineString<f64> = LineString(vec![
            Coord { x: -3.0, y: 0.0 },
            Coord { x: -2.0, y: 1.0 },
            Coord { x: -1.0, y: 2.0 },
            Coord { x: 0.0, y: 3.0 },
            Coord { x: 1.0, y: 2.0 },
            Coord { x: 2.0, y: 1.0 },
            Coord { x: 3.0, y: 0.0 },
            Coord { x: 1.0, y: 0.0 },
            Coord { x: -1.0, y: 0.0 },
            Coord { x: -3.0, y: 0.0 },
        ]);
        let correct_polygon = Polygon::new(correct_ext, vec![correct_int]);
        let max_dist = 2.0;
        let densified = polygon.densify(max_dist);
        assert_eq!(densified, correct_polygon);
    }

    #[test]
    fn test_empty_linestring_densify() {
        let linestring = LineString::<f64>::new(vec![]);
        let max_dist = 2.0;
        let densified = linestring.densify(max_dist);
        assert!(densified.0.is_empty());
    }

    #[test]
    fn test_linestring_densify() {
        let linestring: LineString<f64> =
            vec![[-1.0, 0.0], [0.0, 0.0], [0.0, 6.0], [1.0, 8.0]].into();
        let correct: LineString<f64> = vec![
            [-1.0, 0.0],
            [0.0, 0.0],
            [0.0, 2.0],
            [0.0, 4.0],
            [0.0, 6.0],
            [0.5, 7.0],
            [1.0, 8.0],
        ]
        .into();
        let max_dist = 2.0;
        let densified = linestring.densify(max_dist);
        assert_eq!(densified, correct);
    }

    #[test]
    fn test_line_densify() {
        let line: Line<f64> = Line::new(coord! {x: 0.0, y: 6.0}, coord! {x: 1.0, y: 8.0});
        let correct: LineString<f64> = vec![[0.0, 6.0], [0.5, 7.0], [1.0, 8.0]].into();
        let max_dist = 2.0;
        let densified = line.densify(max_dist);
        assert_eq!(densified, correct);
    }
}
