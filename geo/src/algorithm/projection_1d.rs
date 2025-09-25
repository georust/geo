use crate::CoordFloat;
use crate::geometry::*;

/// Project polygon geometry from 2D space into 1D space.
///
/// This trait provides methods to collapse polygon points onto a single axis,
/// while preserving references to the original 2D coordinates. This enables
/// spatial indexing techniques where the 1D projection is used to identify
/// candidate closest points, then true euclidean distances are calculated
/// using the original coordinates.
pub(crate) trait Project1D<T>
where
    T: CoordFloat,
{
    /// Project all polygon points onto the X-axis.
    ///
    /// Returns a vector of tuples where:
    /// - First element: x-coordinate (projected value)
    /// - Second element: index into the original polygon's vertex array
    #[allow(dead_code)]
    fn project_x(&self) -> Vec<(T, usize)>;

    /// Project all polygon points onto the Y-axis.
    ///
    /// Returns a vector of tuples where:
    /// - First element: y-coordinate (projected value)
    /// - Second element: index into the original polygon's vertex array
    #[allow(dead_code)]
    fn project_y(&self) -> Vec<(T, usize)>;
}

impl<T> Project1D<T> for Polygon<T>
where
    T: CoordFloat,
{
    fn project_x(&self) -> Vec<(T, usize)> {
        let mut projections = Vec::with_capacity(self.exterior().0.len());
        let mut global_idx = 0;

        // Project exterior ring
        for coord in self.exterior().coords() {
            projections.push((coord.x, global_idx));
            global_idx += 1;
        }

        // Project interior rings (holes)
        for interior in self.interiors() {
            for coord in interior.coords() {
                projections.push((coord.x, global_idx));
                global_idx += 1;
            }
        }

        projections
    }

    fn project_y(&self) -> Vec<(T, usize)> {
        let mut projections = Vec::with_capacity(self.exterior().0.len());
        let mut global_idx = 0;

        // Project exterior ring
        for coord in self.exterior().coords() {
            projections.push((coord.y, global_idx));
            global_idx += 1;
        }

        // Project interior rings (holes)
        for interior in self.interiors() {
            for coord in interior.coords() {
                projections.push((coord.y, global_idx));
                global_idx += 1;
            }
        }

        projections
    }
}

impl<T> Project1D<T> for MultiPolygon<T>
where
    T: CoordFloat,
{
    fn project_x(&self) -> Vec<(T, usize)> {
        let mut projections = Vec::new();
        let mut global_index = 0;

        // Iterate through all polygons and project each one
        for polygon in self.iter() {
            // Project exterior ring
            for coord in polygon.exterior().coords() {
                projections.push((coord.x, global_index));
                global_index += 1;
            }

            // Project interior rings (holes)
            for interior in polygon.interiors() {
                for coord in interior.coords() {
                    projections.push((coord.x, global_index));
                    global_index += 1;
                }
            }
        }

        projections
    }

    fn project_y(&self) -> Vec<(T, usize)> {
        let mut projections = Vec::new();
        let mut global_index = 0;

        // Iterate through all polygons and project each one
        for polygon in self.iter() {
            // Project exterior ring
            for coord in polygon.exterior().coords() {
                projections.push((coord.y, global_index));
                global_index += 1;
            }

            // Project interior rings (holes)
            for interior in polygon.interiors() {
                for coord in interior.coords() {
                    projections.push((coord.y, global_index));
                    global_index += 1;
                }
            }
        }

        projections
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{coord, polygon};

    #[test]
    fn test_polygon_project_x() {
        let poly = polygon![
            (x: 0.0, y: 0.0),
            (x: 4.0, y: 0.0),
            (x: 4.0, y: 4.0),
            (x: 0.0, y: 4.0),
            (x: 0.0, y: 0.0),
        ];

        let projected = poly.project_x();
        let expected = vec![
            (0.0, 0), // x=0.0, index=0
            (4.0, 1), // x=4.0, index=1
            (4.0, 2), // x=4.0, index=2
            (0.0, 3), // x=0.0, index=3
            (0.0, 4), // x=0.0, index=4
        ];

        assert_eq!(projected, expected);
    }

    #[test]
    fn test_polygon_project_y() {
        let poly = polygon![
            (x: 0.0, y: 0.0),
            (x: 4.0, y: 0.0),
            (x: 4.0, y: 4.0),
            (x: 0.0, y: 4.0),
            (x: 0.0, y: 0.0),
        ];

        let projected = poly.project_y();
        let expected = vec![
            (0.0, 0), // y=0.0, index=0
            (0.0, 1), // y=0.0, index=1
            (4.0, 2), // y=4.0, index=2
            (4.0, 3), // y=4.0, index=3
            (0.0, 4), // y=0.0, index=4
        ];

        assert_eq!(projected, expected);
    }

    #[test]
    fn test_polygon_with_hole_project_x() {
        let poly = Polygon::new(
            LineString::new(vec![
                coord! { x: 0.0, y: 0.0 },
                coord! { x: 10.0, y: 0.0 },
                coord! { x: 10.0, y: 10.0 },
                coord! { x: 0.0, y: 10.0 },
                coord! { x: 0.0, y: 0.0 },
            ]),
            vec![LineString::new(vec![
                coord! { x: 2.0, y: 2.0 },
                coord! { x: 8.0, y: 2.0 },
                coord! { x: 8.0, y: 8.0 },
                coord! { x: 2.0, y: 8.0 },
                coord! { x: 2.0, y: 2.0 },
            ])],
        );

        let projected = poly.project_x();
        assert_eq!(projected.len(), 10); // 5 exterior + 5 interior points

        // Check that x-coordinates are preserved as projection values
        assert_eq!(projected[0].0, 0.0);
        assert_eq!(projected[1].0, 10.0);
        assert_eq!(projected[5].0, 2.0);
        assert_eq!(projected[6].0, 8.0);

        // Check that indices are correct
        assert_eq!(projected[0].1, 0); // first exterior vertex
        assert_eq!(projected[1].1, 1); // second exterior vertex
        assert_eq!(projected[5].1, 5); // first interior vertex (offset by exterior length)
        assert_eq!(projected[6].1, 6); // second interior vertex
    }

    #[test]
    fn test_multi_polygon_project_x() {
        let multi_poly = MultiPolygon::new(vec![
            polygon![
                (x: 0.0, y: 0.0),
                (x: 2.0, y: 0.0),
                (x: 2.0, y: 2.0),
                (x: 0.0, y: 2.0),
                (x: 0.0, y: 0.0),
            ],
            polygon![
                (x: 5.0, y: 5.0),
                (x: 7.0, y: 5.0),
                (x: 7.0, y: 7.0),
                (x: 5.0, y: 7.0),
                (x: 5.0, y: 5.0),
            ],
        ]);

        let projected = multi_poly.project_x();
        assert_eq!(projected.len(), 10); // 5 points from each polygon

        // Check first polygon's x-coordinates
        assert_eq!(projected[0].0, 0.0);
        assert_eq!(projected[1].0, 2.0);

        // Check second polygon's x-coordinates
        assert_eq!(projected[5].0, 5.0);
        assert_eq!(projected[6].0, 7.0);

        // Check that indices are correct
        assert_eq!(projected[0].1, 0); // first vertex of first polygon
        assert_eq!(projected[5].1, 5); // first vertex of second polygon
    }

    #[test]
    fn test_multi_polygon_project_y() {
        let multi_poly = MultiPolygon::new(vec![
            polygon![
                (x: 0.0, y: 0.0),
                (x: 2.0, y: 0.0),
                (x: 2.0, y: 2.0),
                (x: 0.0, y: 2.0),
                (x: 0.0, y: 0.0),
            ],
            polygon![
                (x: 5.0, y: 5.0),
                (x: 7.0, y: 5.0),
                (x: 7.0, y: 7.0),
                (x: 5.0, y: 7.0),
                (x: 5.0, y: 5.0),
            ],
        ]);

        let projected = multi_poly.project_y();
        assert_eq!(projected.len(), 10); // 5 points from each polygon

        // Check first polygon's y-coordinates
        assert_eq!(projected[0].0, 0.0);
        assert_eq!(projected[2].0, 2.0);

        // Check second polygon's y-coordinates
        assert_eq!(projected[5].0, 5.0);
        assert_eq!(projected[7].0, 7.0);

        // Check that indices are correct
        assert_eq!(projected[0].1, 0); // first vertex of first polygon
        assert_eq!(projected[5].1, 5); // first vertex of second polygon
    }
}
