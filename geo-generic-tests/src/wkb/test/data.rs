use geo_types::{
    line_string, point, polygon, Geometry, GeometryCollection, LineString, MultiLineString,
    MultiPoint, MultiPolygon, Point, Polygon,
};

pub(super) fn point_2d() -> Point {
    point!(
        x: 0., y: 1.
    )
}

pub(super) fn linestring_2d() -> LineString {
    line_string![
        (x: 0., y: 1.),
        (x: 1., y: 2.)
    ]
}

pub(super) fn polygon_2d() -> Polygon {
    polygon![
        (x: -111., y: 45.),
        (x: -111., y: 41.),
        (x: -104., y: 41.),
        (x: -104., y: 45.),
    ]
}

pub(super) fn polygon_2d_with_interior() -> Polygon {
    polygon!(
        exterior: [
            (x: -111., y: 45.),
            (x: -111., y: 41.),
            (x: -104., y: 41.),
            (x: -104., y: 45.),
        ],
        interiors: [
            [
                (x: -110., y: 44.),
                (x: -110., y: 42.),
                (x: -105., y: 42.),
                (x: -105., y: 44.),
            ],
        ],
    )
}

pub(super) fn multi_point_2d() -> MultiPoint {
    MultiPoint::new(vec![
        point!(
            x: 0., y: 1.
        ),
        point!(
            x: 1., y: 2.
        ),
    ])
}

pub(super) fn multi_line_string_2d() -> MultiLineString {
    MultiLineString::new(vec![
        line_string![
            (x: -111., y: 45.),
            (x: -111., y: 41.),
            (x: -104., y: 41.),
            (x: -104., y: 45.),
        ],
        line_string![
            (x: -110., y: 44.),
            (x: -110., y: 42.),
            (x: -105., y: 42.),
            (x: -105., y: 44.),
        ],
    ])
}

pub(super) fn multi_polygon_2d() -> MultiPolygon {
    MultiPolygon::new(vec![
        polygon![
            (x: -111., y: 45.),
            (x: -111., y: 41.),
            (x: -104., y: 41.),
            (x: -104., y: 45.),
        ],
        polygon!(
            exterior: [
                (x: -111., y: 45.),
                (x: -111., y: 41.),
                (x: -104., y: 41.),
                (x: -104., y: 45.),
            ],
            interiors: [
                [
                    (x: -110., y: 44.),
                    (x: -110., y: 42.),
                    (x: -105., y: 42.),
                    (x: -105., y: 44.),
                ],
            ],
        ),
    ])
}

pub(super) fn geometry_collection_2d() -> GeometryCollection {
    GeometryCollection::new_from(vec![
        Geometry::Point(point_2d()),
        Geometry::LineString(linestring_2d()),
        Geometry::Polygon(polygon_2d()),
        Geometry::Polygon(polygon_2d_with_interior()),
        Geometry::MultiPoint(multi_point_2d()),
        Geometry::MultiLineString(multi_line_string_2d()),
        Geometry::MultiPolygon(multi_polygon_2d()),
    ])
}
