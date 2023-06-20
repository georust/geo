/// Iterate over a slice in overlapping pairs
///
/// # Examples
///
/// ```ignore
/// # use crate::offset::slice_itertools::pairwise;
/// let input = vec![1, 2, 3, 4, 5];
/// let output_actual: Vec<(i32, i32)> =
///     pairwise(&input[..]).map(|(a, b)| (*a, *b)).collect();
/// let output_expected = vec![(1, 2), (2, 3), (3, 4), (4, 5)];
/// assert_eq!(
///     output_actual,
///     output_expected
/// )
/// ```
///
/// # Note
///
/// We already have [std::slice::windows()] but the problem is it returns a
/// slice, not a tuple; and therefore it is not easy to unpack the result since
/// a slice cannot be used as an irrefutable pattern. For example, the `.map()`
/// in the following snippet creates a compiler error something like `Refutable
/// pattern in function argument; options &[_] and &[_,_,..] are not covered.`
///
/// ```ignore
/// let some_vector:Vec<i64> = vec![1,2,3];
/// let some_slice:&[i64] = &some_vector[..];
/// let some_result:Vec<i64> = some_slice
///     .windows(2)
///     .map(|&[a, b]| a + b) // <-- error
///     .collect();
/// ```
///
pub(super) fn pairwise<T>(
    slice: &[T],
) -> std::iter::Zip<std::slice::Iter<T>, std::slice::Iter<T>> {
    if slice.len() == 0 {
        // The following nonsense is needed because slice[1..] would panic
        // and because std::iter::empty returns a new type which is super annoying
        // fingers crossed the compiler will optimize this out anyway
        [].iter().zip([].iter())
    } else {
        slice.iter().zip(slice[1..].iter())
    }
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
    fn test_pairwise_one_element() {
        let items = vec![1];
        let actual_result: Vec<(i32, i32)> = pairwise(&items[..]).map(|(a, b)| (*a, *b)).collect();
        let expected_result = vec![];
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn test_pairwise_zero_elements() {
        let items = vec![];
        let actual_result: Vec<(i32, i32)> = pairwise(&items[..]).map(|(a, b)| (*a, *b)).collect();
        let expected_result = vec![];
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
