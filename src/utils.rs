use std::{collections::HashSet, hash::Hash};

pub fn vec_eq_unordered<T, const N: usize>(v1: &[T], v2: [T; N]) -> bool
where
    T: Eq + Hash + Copy,
{
    v1.iter().copied().collect::<HashSet<_>>() == HashSet::from(v2)
}

pub trait FromRef<T> {
    fn from_ref(t: &T) -> Self;
}

pub trait Pick<T> {
    fn pick(&self) -> T;
}

impl<T, U> Pick<U> for T
where
    U: FromRef<T>,
{
    fn pick(&self) -> U {
        U::from_ref(self)
    }
}
