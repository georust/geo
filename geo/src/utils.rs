//! Internal utility functions, types, and data structures.

/// Partition a mutable slice in-place so that it contains all elements for
/// which `predicate(e)` is `true`, followed by all elements for which
/// `predicate(e)` is `false`. Returns sub-slices to all predicated and
/// non-predicated elements, respectively.
///
/// https://github.com/llogiq/partition/blob/master/src/lib.rs
pub fn partition_slice<T, P>(data: &mut [T], predicate: P) -> (&mut [T], &mut [T])
where
    P: Fn(&T) -> bool,
{
    let len = data.len();
    if len == 0 {
        return (&mut [], &mut []);
    }
    let (mut l, mut r) = (0, len - 1);
    loop {
        while l < len && predicate(&data[l]) {
            l += 1;
        }
        while r > 0 && !predicate(&data[r]) {
            r -= 1;
        }
        if l >= r {
            return data.split_at_mut(l);
        }
        data.swap(l, r);
    }
}
