use crate::{
    CoordFloat, CoordTZM, GeometryCollectionTZM, GeometryTZM, LineStringTZM, Measure,
    MultiLineStringTZM, MultiPointTZM, MultiPolygonTZM, PointTZM, PolygonTZM, RectTZM, TriangleTZM,
    ZCoord,
};
use std::mem;

impl<'a, T, Z, M> arbitrary::Arbitrary<'a> for CoordTZM<T, Z, M>
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

impl<'a, T, Z, M> arbitrary::Arbitrary<'a> for PointTZM<T, Z, M>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
    Z: arbitrary::Arbitrary<'a> + ZCoord,
    M: arbitrary::Arbitrary<'a> + Measure,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary::<CoordTZM<T, Z, M>>().map(Self)
    }
}

impl<'a, T, Z, M> arbitrary::Arbitrary<'a> for LineStringTZM<T, Z, M>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
    Z: arbitrary::Arbitrary<'a> + ZCoord,
    M: arbitrary::Arbitrary<'a> + Measure,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let coords = u.arbitrary::<Vec<CoordTZM<T, Z, M>>>()?;
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

impl<'a, T, Z, M> arbitrary::Arbitrary<'a> for PolygonTZM<T, Z, M>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
    Z: arbitrary::Arbitrary<'a> + ZCoord,
    M: arbitrary::Arbitrary<'a> + Measure,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self::new(
            u.arbitrary::<LineStringTZM<T, Z, M>>()?,
            u.arbitrary::<Vec<LineStringTZM<T, Z, M>>>()?,
        ))
    }
}

impl<'a, T, Z, M> arbitrary::Arbitrary<'a> for MultiPointTZM<T, Z, M>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
    Z: arbitrary::Arbitrary<'a> + ZCoord,
    M: arbitrary::Arbitrary<'a> + Measure,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary::<Vec<PointTZM<T, Z, M>>>().map(Self)
    }
}

impl<'a, T, Z, M> arbitrary::Arbitrary<'a> for MultiLineStringTZM<T, Z, M>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
    Z: arbitrary::Arbitrary<'a> + ZCoord,
    M: arbitrary::Arbitrary<'a> + Measure,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary::<Vec<LineStringTZM<T, Z, M>>>().map(Self)
    }
}

impl<'a, T, Z, M> arbitrary::Arbitrary<'a> for MultiPolygonTZM<T, Z, M>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
    Z: arbitrary::Arbitrary<'a> + ZCoord,
    M: arbitrary::Arbitrary<'a> + Measure,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary::<Vec<PolygonTZM<T, Z, M>>>().map(Self)
    }
}

impl<'a, T, Z, M> arbitrary::Arbitrary<'a> for GeometryCollectionTZM<T, Z, M>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
    Z: arbitrary::Arbitrary<'a> + ZCoord,
    M: arbitrary::Arbitrary<'a> + Measure,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary()
    }
}

impl<'a, T, Z, M> arbitrary::Arbitrary<'a> for RectTZM<T, Z, M>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
    Z: arbitrary::Arbitrary<'a> + ZCoord,
    M: arbitrary::Arbitrary<'a> + Measure,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self::new(
            u.arbitrary::<CoordTZM<T, Z, M>>()?,
            u.arbitrary::<CoordTZM<T, Z, M>>()?,
        ))
    }
}

impl<'a, T, Z, M> arbitrary::Arbitrary<'a> for TriangleTZM<T, Z, M>
where
    T: arbitrary::Arbitrary<'a> + CoordFloat,
    Z: arbitrary::Arbitrary<'a> + ZCoord,
    M: arbitrary::Arbitrary<'a> + Measure,
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self(
            u.arbitrary::<CoordTZM<T, Z, M>>()?,
            u.arbitrary::<CoordTZM<T, Z, M>>()?,
            u.arbitrary::<CoordTZM<T, Z, M>>()?,
        ))
    }
}

impl<'a, T, Z, M> arbitrary::Arbitrary<'a> for GeometryTZM<T, Z, M>
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
