use std::{collections::HashSet, hash::Hash};

pub fn vec_eq_unordered<T, const N: usize>(v1: &[T], v2: [T; N]) -> bool
where
    T: Eq + Hash + Copy,
{
    v1.iter().copied().collect::<HashSet<_>>() == HashSet::from(v2)
}
