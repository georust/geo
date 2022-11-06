use crate::{
    Coord, CoordFloat, Geometry, GeometryCollection, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, Polygon, Rect, Triangle,
};
use std::mem;

impl<'a, T> arbitrary::Arbitrary<'a> for Coord<T>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(coord! {
            x: u.arbitrary::<T>()?,
            y: u.arbitrary::<T>()?,
        })
    }
}

impl<'a, T> arbitrary::Arbitrary<'a> for Point<T>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary::<Coord<T>>().map(Self)
    }
}

impl<'a, T> arbitrary::Arbitrary<'a> for LineString<T>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let coords = u.arbitrary::<Vec<Coord<T>>>()?;
        if coords.len() < 2 {
            Err(arbitrary::Error::IncorrectFormat)
        } else {
            Ok(Self(coords))
        }
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        (mem::size_of::<T>() * 2, None)
    }
}

impl<'a, T> arbitrary::Arbitrary<'a> for Polygon<T>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self::new(
            u.arbitrary::<LineString<T>>()?,
            u.arbitrary::<Vec<LineString<T>>>()?,
        ))
    }
}

impl<'a, T> arbitrary::Arbitrary<'a> for MultiPoint<T>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary::<Vec<Point<T>>>().map(Self)
    }
}

impl<'a, T> arbitrary::Arbitrary<'a> for MultiLineString<T>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary::<Vec<LineString<T>>>().map(Self)
    }
}

impl<'a, T> arbitrary::Arbitrary<'a> for MultiPolygon<T>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary::<Vec<Polygon<T>>>().map(Self)
    }
}

impl<'a, T> arbitrary::Arbitrary<'a> for GeometryCollection<T>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary()
    }
}

impl<'a, T> arbitrary::Arbitrary<'a> for Rect<T>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self::new(
            u.arbitrary::<Coord<T>>()?,
            u.arbitrary::<Coord<T>>()?,
        ))
    }
}

impl<'a, T> arbitrary::Arbitrary<'a> for Triangle<T>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self(
            u.arbitrary::<Coord<T>>()?,
            u.arbitrary::<Coord<T>>()?,
            u.arbitrary::<Coord<T>>()?,
        ))
    }
}

impl<'a, T> arbitrary::Arbitrary<'a> for Geometry<T>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let n = u.int_in_range(0..=8)?;

        Ok(match n {
            0 => Self::Point(u.arbitrary()?),
            1 => Self::LineString(u.arbitrary()?),
            2 => Self::Polygon(u.arbitrary()?),
            3 => Self::MultiPoint(u.arbitrary()?),
            4 => Self::MultiLineString(u.arbitrary()?),
            5 => Self::MultiPolygon(u.arbitrary()?),
            6 => Self::GeometryCollection(u.arbitrary()?),
            7 => Self::Triangle(u.arbitrary()?),
            8 => Self::Rect(u.arbitrary()?),
            _ => unreachable!(),
        })
    }
}
