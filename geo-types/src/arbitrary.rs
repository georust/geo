use crate::{
    CoordFloat, Coordinate, Geometry, GeometryCollection, LineString, Measure, MultiLineString,
    MultiPoint, MultiPolygon, Point, Polygon, Rect, Triangle, ZCoord,
};
use std::mem;

impl<'a, T, Z, M> arbitrary::Arbitrary<'a> for Coordinate<T, Z, M>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
    Z: arbitrary::Arbitrary<'a> + ZCoord,
    M: arbitrary::Arbitrary<'a> + Measure,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(coord! {
            x: u.arbitrary::<T>()?,
            y: u.arbitrary::<T>()?,
            z: u.arbitrary::<Z>()?,
            m: u.arbitrary::<M>()?,
        })
    }
}

impl<'a, T, Z, M> arbitrary::Arbitrary<'a> for Point<T, Z, M>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
    Z: arbitrary::Arbitrary<'a> + ZCoord,
    M: arbitrary::Arbitrary<'a> + Measure,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary::<Coordinate<T, Z, M>>().map(Self)
    }
}

impl<'a, T, Z, M> arbitrary::Arbitrary<'a> for LineString<T, Z, M>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
    Z: arbitrary::Arbitrary<'a> + ZCoord,
    M: arbitrary::Arbitrary<'a> + Measure,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let coords = u.arbitrary::<Vec<Coordinate<T, Z, M>>>()?;
        if coords.len() < 2 {
            Err(arbitrary::Error::IncorrectFormat)
        } else {
            Ok(Self(coords))
        }
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        (
            mem::size_of::<T>() * 2 + mem::size_of::<Z>() + mem::size_of::<M>(),
            None,
        )
    }
}

impl<'a, T, Z, M> arbitrary::Arbitrary<'a> for Polygon<T, Z, M>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
    Z: arbitrary::Arbitrary<'a> + ZCoord,
    M: arbitrary::Arbitrary<'a> + Measure,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self::new(
            u.arbitrary::<LineString<T, Z, M>>()?,
            u.arbitrary::<Vec<LineString<T, Z, M>>>()?,
        ))
    }
}

impl<'a, T, Z, M> arbitrary::Arbitrary<'a> for MultiPoint<T, Z, M>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
    Z: arbitrary::Arbitrary<'a> + ZCoord,
    M: arbitrary::Arbitrary<'a> + Measure,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary::<Vec<Point<T, Z, M>>>().map(Self)
    }
}

impl<'a, T, Z, M> arbitrary::Arbitrary<'a> for MultiLineString<T, Z, M>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
    Z: arbitrary::Arbitrary<'a> + ZCoord,
    M: arbitrary::Arbitrary<'a> + Measure,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary::<Vec<LineString<T, Z, M>>>().map(Self)
    }
}

impl<'a, T, Z, M> arbitrary::Arbitrary<'a> for MultiPolygon<T, Z, M>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
    Z: arbitrary::Arbitrary<'a> + ZCoord,
    M: arbitrary::Arbitrary<'a> + Measure,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary::<Vec<Polygon<T, Z, M>>>().map(Self)
    }
}

impl<'a, T, Z, M> arbitrary::Arbitrary<'a> for GeometryCollection<T, Z, M>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
    Z: arbitrary::Arbitrary<'a> + ZCoord,
    M: arbitrary::Arbitrary<'a> + Measure,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary()
    }
}

impl<'a, T, Z, M> arbitrary::Arbitrary<'a> for Rect<T, Z, M>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
    Z: arbitrary::Arbitrary<'a> + ZCoord,
    M: arbitrary::Arbitrary<'a> + Measure,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self::new(
            u.arbitrary::<Coordinate<T, Z, M>>()?,
            u.arbitrary::<Coordinate<T, Z, M>>()?,
        ))
    }
}

impl<'a, T, Z, M> arbitrary::Arbitrary<'a> for Triangle<T, Z, M>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
    Z: arbitrary::Arbitrary<'a> + ZCoord,
    M: arbitrary::Arbitrary<'a> + Measure,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self(
            u.arbitrary::<Coordinate<T, Z, M>>()?,
            u.arbitrary::<Coordinate<T, Z, M>>()?,
            u.arbitrary::<Coordinate<T, Z, M>>()?,
        ))
    }
}

impl<'a, T, Z, M> arbitrary::Arbitrary<'a> for Geometry<T, Z, M>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
    Z: arbitrary::Arbitrary<'a> + ZCoord,
    M: arbitrary::Arbitrary<'a> + Measure,
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
