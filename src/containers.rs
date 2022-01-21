
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

pub trait CardSet<T>: Addable<T> + Removeable<T> {}