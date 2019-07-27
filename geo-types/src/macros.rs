/// Creates a [`Point`] from the given coordinates.
///
/// ```txt
/// point!(«(x,y)»)
/// ```
///
/// # Examples
///
/// Creating a [`Point`], supplying x/y values:
///
/// ```
/// use geo_types::point;
///
/// let p = point!(x: 181.2, y: 51.79);
///
/// assert_eq!(p, geo_types::Point(geo_types::Coordinate {
///     x: 181.2,
///     y: 51.79,
/// }));
/// ```
///
/// [`Point`]: ./struct.Point.html
#[macro_export]
macro_rules! point {
    (x: $x:expr, y: $y:expr) => {
        $crate::Point::new($x, $y)
    };
}

/// Creates a [`LineString`] containing the given coordinates.
///
/// ```txt
/// line_string![«Coordinate|(x,y)», …]
/// ```
///
/// # Examples
///
/// Creating a [`LineString`], supplying x/y values:
///
/// ```
/// use geo_types::line_string;
///
/// let ls = line_string![
///     (x: -21.95156, y: 64.1446),
///     (x: -21.951, y: 64.14479),
///     (x: -21.95044, y: 64.14527),
///     (x: -21.951445, y: 64.145508)
/// ];
///
/// assert_eq!(ls[1], geo_types::Coordinate {
///     x: -21.951,
///     y: 64.14479
/// });
/// ```
///
/// Creating a [`LineString`], supplying [`Coordinate`]s:
///
/// ```
/// use geo_types::line_string;
///
/// let coord1 = geo_types::Coordinate { x: -21.95156, y: 64.1446 };
/// let coord2 = geo_types::Coordinate { x: -21.951, y: 64.14479 };
/// let coord3 = geo_types::Coordinate { x: -21.95044, y: 64.14527 };
/// let coord4 = geo_types::Coordinate { x: -21.951445, y: 64.145508 };
///
/// let ls = line_string![coord1, coord2, coord3, coord4];
///
/// assert_eq!(ls[1], geo_types::Coordinate {
///     x: -21.951,
///     y: 64.14479
/// });
/// ```
///
/// [`Coordinate`]: ./struct.Coordinate.html
/// [`LineString`]: ./struct.LineString.html
#[macro_export]
macro_rules! line_string {
    () => { $crate::LineString(vec![]) };
    (
        $((x: $x:expr, y: $y:expr)),*
        $(,)?
    ) => {
        line_string![
            $(
                $crate::Coordinate { x: $x, y: $y },
            )*
        ]
    };
    (
        $($coord:expr),*
        $(,)?
    ) => {
        $crate::LineString(
            <[_]>::into_vec(
                ::std::boxed::Box::new(
                    [$($coord), *]
                )
            )
        )
    };
}

/// Creates a [`Polygon`] containing the given coordinates.
///
/// ```txt
/// polygon![«Coordinate|(x,y)», …]
/// // or
/// polygon!(
///     exterior: [«Coordinate|(x,y)», …],
///     interiors: [
///         [«Coordinate|(x,y)», …],
///         …
///     ],
/// )
/// ```
///
/// # Examples
///
/// Creating a [`Polygon`] without interior rings, supplying x/y values:
///
/// ```
/// use geo_types::polygon;
///
/// let poly = polygon![
///     (x: -111., y: 45.),
///     (x: -111., y: 41.),
///     (x: -104., y: 41.),
///     (x: -104., y: 45.),
/// ];
///
/// assert_eq!(
///     poly.exterior()[1],
///     geo_types::Coordinate { x: -111., y: 41. },
/// );
/// ```
///
/// Creating a [`Polygon`], supplying x/y values:
///
/// ```
/// use geo_types::polygon;
///
/// let poly = polygon!(
///     exterior: [
///         (x: -111., y: 45.),
///         (x: -111., y: 41.),
///         (x: -104., y: 41.),
///         (x: -104., y: 45.),
///     ],
///     interiors: [
///         [
///             (x: -110., y: 44.),
///             (x: -110., y: 42.),
///             (x: -105., y: 42.),
///             (x: -105., y: 44.),
///         ],
///     ],
/// );
///
/// assert_eq!(
///     poly.exterior()[1],
///     geo_types::Coordinate { x: -111., y: 41. },
/// );
/// ```
///
/// [`Coordinate`]: ./struct.Coordinate.html
/// [`Polygon`]: ./struct.Polygon.html
#[macro_export]
macro_rules! polygon {
    () => { $crate::Polygon::new(line_string![], vec![]) };
    (
        exterior: [
            $((x: $exterior_x:expr, y: $exterior_y:expr)),*
            $(,)?
        ],
        interiors: [
            $([
                $((x: $interior_x:expr, y: $interior_y:expr)),*
                $(,)?
            ]),*
            $(,)?
        ]
        $(,)?
    ) => {
        polygon!(
            exterior: [
                $(
                    $crate::Coordinate { x: $exterior_x, y: $exterior_y },
                )*
            ],
            interiors: [
                $([
                    $($crate::Coordinate { x: $interior_x, y: $interior_y }),*
                ]),*

            ],
        )
    };
    (
        exterior: [
            $($exterior_coord:expr),*
            $(,)?
        ],
        interiors: [
            $([
                $($interior_coord:expr),*
                $(,)?
            ]),*
            $(,)?
        ]
        $(,)?
    ) => {
        $crate::Polygon::new(
            $crate::line_string![
                $($exterior_coord), *
            ],
            <[_]>::into_vec(
                ::std::boxed::Box::new(
                    [
                        $(
                            $crate::line_string![$($interior_coord),*]
                        ), *
                    ]
                )
            )
        )
    };
    (
        $((x: $x:expr, y: $y:expr)),*
        $(,)?
    ) => {
        polygon![
            $($crate::Coordinate { x: $x, y: $y }),*
        ]
    };
    (
        $($coord:expr),*
        $(,)?
    ) => {
        $crate::Polygon::new(
            $crate::line_string![$($coord,)*],
            vec![],
        )
    };
}
