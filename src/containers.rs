use crate::card::{Achievement, Card};
use crate::enums::Icon;
use std::cell::RefCell;
use std::ops::Deref;

pub trait Addable<T> {
    fn add(&mut self, elem: T);

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

pub trait Removeable<T, P> {
    fn remove(&mut self, param: &P) -> Option<T>;
}

pub trait CardSet<'a, T: 'a>: Addable<&'a T> + Removeable<&'a T, T> {
    fn as_vec(&'_ self) -> Vec<&'a T>;
    fn as_iter(&self) -> Box<dyn Iterator<Item = &'a T> + 'a> {
        Box::new(self.as_vec().into_iter())
    }
}

impl<'a, T> Addable<&'a T> for Box<dyn CardSet<'a, T> + 'a> {
    fn add(&mut self, elem: &'a T) {
        (**self).add(elem)
    }
}

impl<'a, T> Removeable<&'a T, T> for Box<dyn CardSet<'a, T> + 'a> {
    fn remove(&mut self, elem: &T) -> Option<&'a T> {
        (**self).remove(elem)
    }
}

impl<'a, 'b, T> dyn CardSet<'a, T> + 'b {
    pub fn filtered_vec<P>(&self, predicate: P) -> Vec<&'a T>
    where
        P: FnMut(&&'a T) -> bool,
    {
        self.as_iter().filter(predicate).collect()
    }
}

impl<'a, 'b> dyn CardSet<'a, Card> + 'b {
    pub fn has_icon(&self, icon: Icon) -> Vec<&'a Card> {
        self.filtered_vec(|&c| c.contains(icon))
    }
}

pub struct VecSet<T> {
    v: Vec<T>,
}

impl<T> VecSet<T> {
    pub fn inner(&self) -> &Vec<T> {
        &self.v
    }

    pub fn try_remove<P>(&mut self, f: P) -> Option<T>
    where
        P: Fn(&T) -> bool,
    {
        let i = self.v.iter().position(f);
        match i {
            Some(v) => Some(self.v.remove(v)),
            None => None,
        }
    }
}

impl<T: Clone> VecSet<T> {
    pub fn clone_inner(&self) -> Vec<T> {
        self.v.clone()
    }
}

impl<T> Default for VecSet<T> {
    fn default() -> VecSet<T> {
        VecSet { v: Vec::new() }
    }
}

impl<T> Addable<T> for VecSet<T> {
    fn add(&mut self, elem: T) {
        self.v.push(elem)
    }
}

impl<'a, T: PartialEq> Removeable<&'a T, T> for VecSet<&'a T> {
    fn remove(&mut self, elem: &T) -> Option<&'a T> {
        let i = self.v.iter().position(|x| *x == elem);
        match i {
            Some(v) => Some(self.v.remove(v)),
            None => None,
        }
    }
}

impl<'a, T: PartialEq> CardSet<'a, T> for VecSet<&'a T> {
    fn as_vec(&self) -> Vec<&'a T> {
        self.clone_inner()
    }
}

pub type BoxCardSet<'a> = Box<dyn CardSet<'a, Card> + 'a>;
pub type BoxAchievementSet<'a> = Box<dyn CardSet<'a, Achievement<'a>> + 'a>;

pub fn transfer<'a, T, P, R, S, A, B>(from: A, to: B, param: &P) -> Option<&'a T>
where
    R: Removeable<&'a T, P>,
    S: Addable<&'a T>,
    A: Deref<Target = RefCell<R>>,
    B: Deref<Target = RefCell<S>>,
{
    let c = from.borrow_mut().remove(param);
    if let Some(card) = c {
        to.borrow_mut().add(card);
    }
    c
}
