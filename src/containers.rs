
pub trait Addable<T> {
    fn add(&self, elem: T);

    fn optional_add(&self, elem: Option<T>) -> bool {
        // return success?
        match elem {
            Some(value) => {
                self.add(value);
                true
            }
            None => false
        }
    }
}

pub trait Removeable<T> {
    fn remove(&self, elem: &T) -> Option<T>; // return ownership
}

pub trait Popable<T> {
    fn pop(&self) -> Option<T>;
}

pub trait CardSet<T>: Addable<T> + Removeable<T> + Default {}

pub struct VecSet<T> {
    v: Vec<T>
}

impl<T> Default for VecSet<T> {
    fn default() -> VecSet<T> {
        VecSet { v: Vec::new() }
    }
}

impl<T> Addable<T> for VecSet<T> {
    fn add(&self, elem: T) {
        self.v.push(elem)
    }
}

impl<T: PartialEq> Removeable<T> for VecSet<T> {
    fn remove(&self, elem: &T) -> Option<T> {
        let i = self.v.iter().position(|x| x == elem);
        match i {
            Some(v) => Some(self.v.remove(v)),
            None => None
        }
    }
}

impl<T: PartialEq> CardSet<T> for VecSet<T> {}