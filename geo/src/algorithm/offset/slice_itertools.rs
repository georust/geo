/// Iterate over a slice in overlapping pairs
///
/// ```ignore
/// let items = vec![1, 2, 3, 4, 5];
/// let actual_result: Vec<(i32, i32)> = 
///     pairwise(&items[..]).map(|(a, b)| (*a, *b)).collect();
/// let expected_result = vec![(1, 2), (2, 3), (3, 4), (4, 5)];
/// ```
pub(super) fn pairwise<T>(slice: &[T]) -> std::iter::Zip<std::slice::Iter<T>, std::slice::Iter<T>> {
    slice.iter().zip(slice[1..].iter())
}

/// Iterate over a slice and repeat the first item at the end
/// 
/// ```ignore
/// let items = vec![1, 2, 3, 4, 5];
/// let actual_result: Vec<i32> = wrap_one(&items[..]).cloned().collect();
/// let expected_result = vec![1, 2, 3, 4, 5, 1];
/// ```
pub(super) fn wrap_one<T>(
    slice: &[T],
) -> std::iter::Chain<std::slice::Iter<T>, std::slice::Iter<T>> {
    slice.iter().chain(slice[..1].iter())
    //.chain::<&T>(std::iter::once(slice[0]))
}

#[cfg(test)]
mod test {
    use super::{pairwise, wrap_one};

    #[test]
    fn test_pairwise() {
        let items = vec![1, 2, 3, 4, 5];
        let actual_result: Vec<(i32, i32)> = pairwise(&items[..]).map(|(a, b)| (*a, *b)).collect();
        let expected_result = vec![(1, 2), (2, 3), (3, 4), (4, 5)];
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn test_wrap() {
        let items = vec![1, 2, 3, 4, 5];
        let actual_result: Vec<i32> = wrap_one(&items[..]).cloned().collect();
        let expected_result = vec![1, 2, 3, 4, 5, 1];
        assert_eq!(actual_result, expected_result);
    }
}
