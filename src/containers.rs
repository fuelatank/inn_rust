pub trait Addable<'a, T> {
    fn add(&mut self, elem: &'a T);

    /*fn optional_add(&mut self, elem: Option<&'a T>) -> bool {
        //
        // return success?
        match elem {
            Some(value) => {
                true
            }
            None => false
        }
    }*/
}

pub trait Removeable<'a, T, P> {
    fn remove(&mut self, param: &P) -> Option<&'a T>;
}

pub trait CardSet<'a, T>: Addable<'a, T> + Removeable<'a, T, T> {
    fn as_vec(&'_ self) -> Vec<&'a T>;
}

impl<'a, T> Addable<'a, T> for Box<dyn CardSet<'a, T>> {
    fn add(&mut self, elem: &'a T) {
        (**self).add(elem)
    }
}

impl<'a, T> Removeable<'a, T, T> for Box<dyn CardSet<'a, T>> {
    fn remove(&mut self, elem: &T) -> Option<&'a T> {
        (**self).remove(elem)
    }
}

pub struct VecSet<'a, T> {
    v: Vec<&'a T>,
}

impl<'a, T> Default for VecSet<'a, T> {
    fn default() -> VecSet<'a, T> {
        VecSet { v: Vec::new() }
    }
}

impl<'a, T> Addable<'a, T> for VecSet<'a, T> {
    fn add(&mut self, elem: &'a T) {
        self.v.push(elem)
    }
}

impl<'a, T: PartialEq> Removeable<'a, T, T> for VecSet<'a, T> {
    fn remove(&mut self, elem: &T) -> Option<&'a T> {
        let i = self.v.iter().position(|x| *x == elem);
        match i {
            Some(v) => Some(self.v.remove(v)),
            None => None,
        }
    }
}

impl<'a, T: PartialEq> CardSet<'a, T> for VecSet<'a, T> {
    fn as_vec(&self) -> Vec<&'a T> {
        self.v.clone()
    }
}
