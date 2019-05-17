use crate::{CoordinateType, Coordinate, Line, LineString, MultiLineString};
use std::collections::HashMap;
use std::convert::From;
use std::hash::Hash;

#[derive(PartialEq, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MapDistanceType {
    NotCalculated,
    Euclidean,
    Haversine,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct EdgeInfo<T>
where
    T: CoordinateType + Eq + Hash
{
    // vector of neighbor nodes that we can go to, stored in the form of (Destination, Distance)
    to: Vec<(Coordinate<T>, Option<T>)>,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GeoMap<T: CoordinateType>
where
    T: CoordinateType + Eq + Hash
{
    inner: HashMap<Coordinate<T>, EdgeInfo<T>>,
    distance_type: MapDistanceType,
}

impl<T> GeoMap<T>
where
    T: CoordinateType + Eq + Hash,
{
    pub fn new<C>(src: C) -> Self
    where
        C: Iterator<Item=Line<T>>
    {
        GeoMap::build(src, None)
    }

    pub fn new_with_size<C>(src: C, size: usize) -> Self
        where
            C: Iterator<Item=Line<T>>
    {
        GeoMap::build(src, Some(size))
    }

    fn build<C>(src: C, cap: Option<usize>) -> Self
        where
            C: Iterator<Item=Line<T>>
    {
        let mut inner: HashMap<Coordinate<T>, EdgeInfo<T>> =
            if let Some(size) = cap {
                HashMap::with_capacity(size)
            } else {
                HashMap::new()
            };

        src
            .for_each(|line| {
                if let Some(edge) = inner.get_mut(&line.start) {
                    edge.to.push((line.end, None));
                    return;
                }

                let mut to = Vec::new();
                to.push((line.end, None));
                inner.insert(line.start, EdgeInfo{ to });
            });

        inner
            .iter_mut()
            .for_each(|val| {
                val.1.to.dedup_by(|a, b| {
                    a.0 == b.0
                });
            });

        GeoMap {
            inner,
            distance_type: MapDistanceType::NotCalculated,
        }
    }
}

impl<T> From<Vec<Line<T>>> for GeoMap<T>
where
    T: CoordinateType + Eq + Hash
{
    fn from(item: Vec<Line<T>>) -> Self {
        let size = item.len();
        assert!(size > 0);

        GeoMap::new_with_size(item.into_iter(), size)
    }
}

impl<T: CoordinateType> From<LineString<T>> for GeoMap<T>
where
    T: CoordinateType + Eq + Hash
{
    fn from(item: LineString<T>) -> Self {
        let size = item.0.len();
        assert!(size > 0);

        let mut it = item.0.into_iter();
        let mut start = it.next().unwrap();

        GeoMap::new_with_size(
            it.map(move |p| {
                let line = Line::new(start, p);
                start = p;
                return line;
            }),
            size - 1
        )
    }
}

impl<T: CoordinateType> From<Vec<LineString<T>>> for GeoMap<T>
where
    T: CoordinateType + Eq + Hash
{
    fn from(item: Vec<LineString<T>>) -> Self {
        assert!(item.len() > 0 && item[0].0.len() > 0);
        let estimated_size = item.len() * item[0].0.len();

        GeoMap::new_with_size(
            item.into_iter().flat_map(|lines| {
                assert!(lines.0.len() > 0);

                let mut it = lines.0.into_iter();
                let mut start = it.next().unwrap();

                it.map(move |p| {
                    let line = Line::new(start, p);
                    start = p;
                    return line;
                })
            }),
            estimated_size
        )
    }
}

impl<T: CoordinateType> From<MultiLineString<T>> for GeoMap<T>
    where
        T: CoordinateType + Eq + Hash
{
    fn from(item: MultiLineString<T>) -> Self {
        GeoMap::from(item.0)
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::{Line, GeoMap};
}