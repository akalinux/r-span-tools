use crate::builder::IncDecCpCmp;
use crate::{
    BlanketIncDecCpCmp, DefaultValues, GetBeginEnd, Mrs, MrsP, RangeRelation,
    first_range_begin_end, last_range_begin_end, next_range_begin_end, previous_range_begin_end,
    range_bounds_to_values, range_relation,
};

use std::borrow::Borrow;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::mem;
use std::ops::RangeBounds;
use std::rc::Rc;

pub struct OverlapIter<
    T,
    V,
    C: IncDecCpCmp<T, V>,
    R: GetBeginEnd<T>,
    L,
    X,
    B: GetBeginEndOption<T, R>,
    Y,
> where
    L: Borrow<RefCell<Vec<R>>>,
    X: Borrow<C>,
{
    src: L,
    step: V,
    cmp: X,
    next: Option<R>,
    back: Option<R>,
    factory: Y,
    _marker: PhantomData<(T, R, C, B)>,
}

pub trait GetBeginEndOption<T, R: GetBeginEnd<T>> {
    fn get_begin_end_opt_factory(&self, opt: Option<(T, T)>) -> Option<R>;
}

pub struct MrsFactory<T> {
    _t: PhantomData<T>,
}

impl<T> MrsFactory<T> {
    pub fn new() -> Self {
        return Self { _t: PhantomData };
    }
}

impl<T> GetBeginEndOption<T, Mrs<T>> for MrsFactory<T> {
    fn get_begin_end_opt_factory(&self, opt: Option<(T, T)>) -> Option<Mrs<T>> {
        match opt {
            Some((a, z)) => Some(Mrs::new(a, z)),
            None => None,
        }
    }
}

impl<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, L, X, B: GetBeginEndOption<T, R>, Y>
    OverlapIter<T, V, C, R, L, X, B, Y>
where
    L: Borrow<RefCell<Vec<R>>>,
    X: Borrow<C>,
    Y: Borrow<B>,
{
    /// Creates a new [crate::OverlapIter] from the slice of R.
    pub fn new(src: L, step: V, cmp: X, factory: Y) -> Self {
        let next = factory
            .borrow()
            .get_begin_end_opt_factory(first_range_begin_end(
                &*src.borrow().borrow(),
                cmp.borrow(),
            ));
        let back = factory
            .borrow()
            .get_begin_end_opt_factory(last_range_begin_end(&*src.borrow().borrow(), cmp.borrow()));
        Self {
            src,
            step,
            cmp,
            next,
            back,
            factory,
            _marker: PhantomData,
        }
    }
}

impl<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, L, X, B: GetBeginEndOption<T, R>, Y> Iterator
    for OverlapIter<T, V, C, R, L, X, B, Y>
where
    L: Borrow<RefCell<Vec<R>>>,
    X: Borrow<C>,
    Y: Borrow<B>,
{
    type Item = R;
    fn next(&mut self) -> Option<Self::Item> {
        let mut next = None;
        if let Some(n) = &self.next {
            match &self.back {
                Some(b) => match range_relation(n, b, self.cmp.borrow()) {
                    RangeRelation::Overlap(_) => {
                        if let Some(begin) = self.cmp.borrow().inc(n.get_end(), &self.step) {
                            next = self.factory.borrow().get_begin_end_opt_factory(
                                next_range_begin_end(
                                    &begin,
                                    &[
                                        MrsP {
                                            r: b,
                                            _t: PhantomData,
                                        },
                                        MrsP {
                                            r: n,
                                            _t: PhantomData,
                                        },
                                    ],
                                    self.cmp.borrow(),
                                ),
                            );
                        }
                    }
                    RangeRelation::Before(_) => {
                        if let Some(begin) = self.cmp.borrow().inc(n.get_end(), &self.step) {
                            next = self.factory.borrow().get_begin_end_opt_factory(
                                next_range_begin_end(
                                    &begin,
                                    &*self.src.borrow().borrow(),
                                    self.cmp.borrow(),
                                ),
                            );
                        }
                    }
                    RangeRelation::After(_) => return None,
                },
                None => (),
            }
        }
        return mem::replace(&mut self.next, next);
    }
}

impl<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, L, X, B: GetBeginEndOption<T, R>, Y>
    DoubleEndedIterator for OverlapIter<T, V, C, R, L, X, B, Y>
where
    L: Borrow<RefCell<Vec<R>>>,
    X: Borrow<C>,
    Y: Borrow<B>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let mut back = None;
        if let Some(b) = &self.back
            && let Some(n) = &self.next
        {
            match range_relation(b, n, self.cmp.borrow()) {
                RangeRelation::Overlap(_) => {
                    if let Some(end) = self.cmp.borrow().dec(b.get_begin(), &self.step) {
                        back = self.factory.borrow().get_begin_end_opt_factory(
                            previous_range_begin_end(
                                &end,
                                &[
                                    MrsP {
                                        r: n,
                                        _t: PhantomData,
                                    },
                                    MrsP {
                                        r: b,
                                        _t: PhantomData,
                                    },
                                ],
                                self.cmp.borrow(),
                            ),
                        );
                    }
                }
                RangeRelation::After(_) => {
                    if let Some(end) = self.cmp.borrow().dec(b.get_begin(), &self.step) {
                        back = self.factory.borrow().get_begin_end_opt_factory(
                            previous_range_begin_end(
                                &end,
                                &*self.src.borrow().borrow(),
                                self.cmp.borrow(),
                            ),
                        );
                    }
                }
                RangeRelation::Before(_) => return None,
            }
        }
        return mem::replace(&mut self.back, back);
    }
}

/// This object acts as a conversion tool for accumulating instances of [std::ops::RangeBounds] and converting them to an internal representation.
/// The Internal representation is used by the [std::iter::IntoIterator] instance that is created to find the most common intersections.
/// This implementation is meant to be generic and can be easily tailored to any data type or structure that requires computing intersections.
pub struct Accumulate<T, V, C: IncDecCpCmp<T, V>, R, B> {
    list: Vec<R>,
    step: V,
    rebound: V,
    cmp: C,
    factory: B,
    _r: PhantomData<(T, R)>,
}

/// This object acts as a conversion tool for accumulating instances of [std::ops::RangeBounds] and converting them to an internal representation.
/// The Internal representation is used by the [std::iter::IntoIterator] instance that is created to find the most common intersections.
/// Unlike [crate::Accumulate], [crate::AccumulateDefaults] works on a set of defaults that work for most primitive number types in rust.
pub struct AccumulateDefaults<T> {
    list: Vec<Mrs<T>>,
    step: T,
    rebound: T,
    cmp: BlanketIncDecCpCmp<T>,
}
impl<T> AccumulateDefaults<T>
where
    BlanketIncDecCpCmp<T>: DefaultValues<T, T>,
{
    pub fn new() -> Self {
        let cmp: BlanketIncDecCpCmp<T> = BlanketIncDecCpCmp::new();
        Self {
            list: Vec::new(),
            step: cmp.default_step(),
            rebound: cmp.default_rebound(),
            cmp,
        }
    }
}

impl<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, B: GetBeginEndOption<T, R>>
    Accumulate<T, V, C, R, B>
{
    pub fn new(list: Vec<R>, step: V, rebound: V, cmp: C, factory: B) -> Self {
        Self {
            list,
            step,
            rebound,
            cmp,
            factory,
            _r: PhantomData,
        }
    }
}

pub trait Accumulator<T, V, C: IncDecCpCmp<T, V>, R> {
    type Cmp;
    fn get_rebound(&self) -> &V;
    fn get_step(&self) -> &V;
    fn add_from_tuple(&mut self, src: (T, T)) -> Option<(usize, &R)>;
    fn set_rebound(&mut self, rebound: V);
    fn set_step(&mut self, step: V);

    fn get_cmp(&self) -> &C;
    fn rebound(&self, r: &impl RangeBounds<T>) -> Option<(T, T)> {
        return range_bounds_to_values(r, self.get_rebound(), self.get_cmp());
    }

    fn add_range(&mut self, r: &impl RangeBounds<T>) -> Option<(usize, &R)> {
        match self.rebound(r) {
            Some(src) => self.add_tuple(src),
            None => None,
        }
    }
    fn add_tuple(&mut self, src: (T, T)) -> Option<(usize, &R)> {
        return self.add_from_tuple(src);
    }
}

impl<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, B: GetBeginEndOption<T, R>>
    Accumulator<T, V, C, R> for Accumulate<T, V, C, R, B>
{
    type Cmp = C;
    fn add_from_tuple(&mut self, src: (T, T)) -> Option<(usize, &R)> {
        if self.cmp.is_invalid_set(&src.0, &src.1) {
            return None;
        }
        match self.factory.get_begin_end_opt_factory(Some(src)) {
            Some(mrs) => {
                self.list.push(mrs);
                let id = self.list.len() - 1;
                return Some((id, &self.list[id]));
            }
            None => None,
        }
    }

    fn get_rebound(&self) -> &V {
        return &self.rebound;
    }

    fn get_step(&self) -> &V {
        return &self.step;
    }

    fn set_rebound(&mut self, rebound: V) {
        self.rebound = rebound;
    }

    fn set_step(&mut self, step: V) {
        self.step = step;
    }
    fn get_cmp(&self) -> &C {
        return &self.cmp;
    }
}

impl<T> Accumulator<T, T, BlanketIncDecCpCmp<T>, Mrs<T>> for AccumulateDefaults<T>
where
    BlanketIncDecCpCmp<T>: DefaultValues<T, T>,
{
    type Cmp = BlanketIncDecCpCmp<T>;

    fn add_from_tuple(&mut self, src: (T, T)) -> Option<(usize, &Mrs<T>)> {
        if IncDecCpCmp::is_invalid_set(&self.cmp, &src.0, &src.1) {
            return None;
        }
        let (a, z) = src;
        let mrs = Mrs::new(a, z);
        self.list.push(mrs);
        let id = self.list.len() - 1;
        return Some((id, &self.list[id]));
    }

    fn get_rebound(&self) -> &T {
        return &self.rebound;
    }

    fn get_step(&self) -> &T {
        return &self.step;
    }

    fn set_rebound(&mut self, rebound: T) {
        self.rebound = rebound;
    }

    fn set_step(&mut self, step: T) {
        self.step = step;
    }
    fn get_cmp(&self) -> &BlanketIncDecCpCmp<T> {
        return &self.cmp;
    }
}

impl<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, B: GetBeginEndOption<T, R>> IntoIterator
    for Accumulate<T, V, C, R, B>
{
    type Item = R;

    type IntoIter = OverlapIter<T, V, C, R, Rc<RefCell<Vec<R>>>, Rc<C>, B, Rc<B>>;

    fn into_iter(self) -> Self::IntoIter {
        return OverlapIter::new(
            Rc::new(RefCell::new(self.list)),
            self.step,
            Rc::new(self.cmp),
            Rc::new(self.factory),
        );
    }
}

impl<T> IntoIterator for AccumulateDefaults<T>
where
    BlanketIncDecCpCmp<T>: DefaultValues<T, T>,
{
    type Item = Mrs<T>;

    type IntoIter = OverlapIter<
        T,
        T,
        BlanketIncDecCpCmp<T>,
        Mrs<T>,
        Rc<RefCell<Vec<Mrs<T>>>>,
        Rc<BlanketIncDecCpCmp<T>>,
        MrsFactory<T>,
        Rc<MrsFactory<T>>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        return OverlapIter::new(
            Rc::new(RefCell::new(self.list)),
            self.step,
            Rc::new(self.cmp),
            Rc::new(MrsFactory::new()),
        );
    }
}
