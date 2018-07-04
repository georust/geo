use {Coordinate, CoordinateType};

#[derive(PartialEq, Clone, Copy, Debug)]
#[cfg_attr(feature = "use-serde", derive(Serialize, Deserialize))]
pub struct Rect<T>
where
    T: CoordinateType,
{
    pub min: Coordinate<T>,
    pub max: Coordinate<T>,
}
