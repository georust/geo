use crate::{
    CoordFloat, Coordinate, Geometry, GeometryCollection, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, Polygon, Rect, Triangle,
};

impl<'a, T: arbitrary::Arbitrary<'a> + CoordFloat> arbitrary::Arbitrary<'a>
    for Coordinate<T>
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let x = u.arbitrary::<T>()?;
        if x.is_nan() {
            return Err(arbitrary::Error::IncorrectFormat);
        }

        let y = u.arbitrary::<T>()?;
        if y.is_nan() {
            return Err(arbitrary::Error::IncorrectFormat);
        }

        Ok(Coordinate { x, y })
    }
}

impl<'a, T: arbitrary::Arbitrary<'a> + CoordFloat> arbitrary::Arbitrary<'a>
    for Point<T>
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary::<Coordinate<T>>().map(Point)
    }
}

impl<'a, T: arbitrary::Arbitrary<'a> + CoordFloat> arbitrary::Arbitrary<'a>
    for LineString<T>
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let coords = u.arbitrary::<Vec<Coordinate<T>>>()?;

        if coords.len() < 2 {
            return Err(arbitrary::Error::IncorrectFormat);
        }

        Ok(LineString(coords))
    }
}

impl<'a, T: arbitrary::Arbitrary<'a> + CoordFloat> arbitrary::Arbitrary<'a>
    for Polygon<T>
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Polygon::new(
            u.arbitrary::<LineString<T>>()?,
            u.arbitrary::<Vec<LineString<T>>>()?,
        ))
    }
}

impl<'a, T: arbitrary::Arbitrary<'a> + CoordFloat> arbitrary::Arbitrary<'a>
    for MultiPoint<T>
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary::<Vec<Point<T>>>().map(MultiPoint)
    }
}

impl<'a, T: arbitrary::Arbitrary<'a> + CoordFloat> arbitrary::Arbitrary<'a>
    for MultiLineString<T>
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary::<Vec<LineString<T>>>().map(MultiLineString)
    }
}

impl<'a, T: arbitrary::Arbitrary<'a> + CoordFloat> arbitrary::Arbitrary<'a>
    for MultiPolygon<T>
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary::<Vec<Polygon<T>>>().map(MultiPolygon)
    }
}

impl<'a, T: arbitrary::Arbitrary<'a> + CoordFloat> arbitrary::Arbitrary<'a>
    for GeometryCollection<T>
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary()
    }
}

impl<'a, T: arbitrary::Arbitrary<'a> + CoordFloat> arbitrary::Arbitrary<'a>
    for Rect<T>
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Rect::new(
            u.arbitrary::<Coordinate<T>>()?,
            u.arbitrary::<Coordinate<T>>()?,
        ))
    }
}

impl<'a, T: arbitrary::Arbitrary<'a> + CoordFloat> arbitrary::Arbitrary<'a>
    for Triangle<T>
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Triangle(
            u.arbitrary::<Coordinate<T>>()?,
            u.arbitrary::<Coordinate<T>>()?,
            u.arbitrary::<Coordinate<T>>()?,
        ))
    }
}

impl<'a, T: arbitrary::Arbitrary<'a> + CoordFloat> arbitrary::Arbitrary<'a>
    for Geometry<T>
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let n = u.int_in_range(0..=8)?;

        Ok(match n {
            0 => Geometry::Point(u.arbitrary()?),
            1 => Geometry::LineString(u.arbitrary()?),
            2 => Geometry::Polygon(u.arbitrary()?),
            3 => Geometry::MultiPoint(u.arbitrary()?),
            4 => Geometry::MultiLineString(u.arbitrary()?),
            5 => Geometry::MultiPolygon(u.arbitrary()?),
            6 => Geometry::GeometryCollection(u.arbitrary()?),
            7 => Geometry::Triangle(u.arbitrary()?),
            8 => Geometry::Rect(u.arbitrary()?),
            _ => unreachable!(),
        })
    }
}
