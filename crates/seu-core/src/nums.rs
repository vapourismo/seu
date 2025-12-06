use crate::product::Cons;
use crate::product::Product;

/// Marker trait for typ-level Peano numbers
pub trait Num: Default {
    /// Runtime value for this type-level number
    const NUM: usize;
}

/// Helper trait for indexing into a product using a type-level number
pub trait Index<Prod: Product>: Num {
    /// Type at the selected index
    type Selected;

    /// Extract only the item at the selected index.
    fn into_selected(self, subject: Prod) -> Self::Selected;

    /// Borrow the item at the selected index.
    fn selected_ref<'a>(&self, subject: &'a Prod) -> &'a Self::Selected;

    /// Mutably borrow the item at the selected index.
    fn selected_mut<'a>(&mut self, subject: &'a mut Prod) -> &'a mut Self::Selected;
}

/// `0`
#[derive(Default)]
pub struct Zero;

impl Num for Zero {
    const NUM: usize = 0;
}

impl<Head, Tail: Product> Index<Cons<Head, Tail>> for Zero {
    type Selected = Head;

    #[inline(always)]
    fn into_selected(self, subject: Cons<Head, Tail>) -> Self::Selected {
        subject.0
    }

    #[inline(always)]
    fn selected_ref<'a>(&self, subject: &'a Cons<Head, Tail>) -> &'a Self::Selected {
        &subject.0
    }

    #[inline(always)]
    fn selected_mut<'a>(&mut self, subject: &'a mut Cons<Head, Tail>) -> &'a mut Self::Selected {
        &mut subject.0
    }
}

/// `1 + Pred`
#[derive(Default)]
pub struct Succ<Pred: Num>(pub Pred);

impl<Pred: Num> Num for Succ<Pred> {
    const NUM: usize = 1 + Pred::NUM;
}

impl<Pred, Head, Tail> Index<Cons<Head, Tail>> for Succ<Pred>
where
    Pred: Index<Tail>,
    Tail: Product,
{
    type Selected = <Pred as Index<Tail>>::Selected;

    #[inline(always)]
    fn into_selected(self, subject: Cons<Head, Tail>) -> Self::Selected {
        self.0.into_selected(subject.1)
    }

    #[inline(always)]
    fn selected_ref<'a>(&self, subject: &'a Cons<Head, Tail>) -> &'a Self::Selected {
        self.0.selected_ref(&subject.1)
    }

    #[inline(always)]
    fn selected_mut<'a>(&mut self, subject: &'a mut Cons<Head, Tail>) -> &'a mut Self::Selected {
        self.0.selected_mut(&mut subject.1)
    }
}
