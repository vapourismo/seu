use core::mem::ManuallyDrop;
use core::ptr;

use crate::nums::Index;
use crate::untagged_sum::UntaggedSum;

mod internal_sum_variants {
    use crate::product::Cons;
    use crate::product::Nil;
    use crate::untagged_sum::UnionFields;
    use crate::untagged_sum::UntaggedSum;

    pub(super) trait SumVariants: UnionFields {
        unsafe fn drop_variant<TopVariants: UnionFields>(
            untagged_sum: &mut UntaggedSum<TopVariants>,
            idx: usize,
        );
    }

    impl SumVariants for Nil {
        #[inline]
        unsafe fn drop_variant<TopVariants: UnionFields>(
            _untagged_sum: &mut UntaggedSum<TopVariants>,
            _idx: usize,
        ) {
        }
    }

    impl<Head, Tail: SumVariants> SumVariants for Cons<Head, Tail> {
        #[inline]
        unsafe fn drop_variant<TopVariants: UnionFields>(
            untagged_sum: &mut UntaggedSum<TopVariants>,
            idx: usize,
        ) {
            unsafe {
                if idx == 0 {
                    untagged_sum.drop_as::<Head>();
                } else {
                    Tail::drop_variant(untagged_sum, idx - 1);
                }
            }
        }
    }
}

/// Mark a product as containing only variant information
#[expect(private_bounds, reason = "We want to hide the internal components")]
pub trait SumVariants: internal_sum_variants::SumVariants {}

impl<T: internal_sum_variants::SumVariants> SumVariants for T {}

/// Tagged union type
pub struct Sum<Variants: SumVariants> {
    index: usize,
    untagged_sum: UntaggedSum<Variants>,
}

impl<Variants: SumVariants> Sum<Variants> {
    /// Construct the sum as the provided variant.
    ///
    /// `Idx` identifies the variant.
    #[inline]
    pub fn new<Idx: Index<Variants>>(index: Idx, data: Idx::Selected) -> Self {
        let data = ManuallyDrop::new(data);
        let untagged_sum = UntaggedSum::new(index, data);

        Self {
            index: Idx::NUM,
            untagged_sum,
        }
    }

    /// Consume the sum using the corresponding handler from `handlers`.
    #[inline]
    pub fn reduce<Output, Handlers>(self, handlers: Handlers) -> Output
    where
        Handlers: Reducer<Variants, Output>,
    {
        let index = self.index;

        let untagged_sum = unsafe {
            let this = ManuallyDrop::new(self);
            ptr::read(&this.untagged_sum)
        };

        unsafe { handlers.reduce(index, untagged_sum) }
    }

    /// Call the appropriate handler from `handlers` using the variant data in this sum.
    #[inline]
    pub fn reduce_ref<'a, Output, Handlers>(&'a self, handlers: Handlers) -> Output
    where
        Handlers: ReducerRef<'a, Variants, Output>,
    {
        unsafe { handlers.reduce_ref(self.index, &self.untagged_sum) }
    }
}

impl<Variants: SumVariants> Drop for Sum<Variants> {
    fn drop(&mut self) {
        // SAFETY: `self.index` is always a valid variant index
        unsafe {
            Variants::drop_variant(&mut self.untagged_sum, self.index);
        }
    }
}

mod internal_reducer {
    use crate::product::Cons;
    use crate::product::Nil;
    use crate::product::Product;
    use crate::sum::SumVariants;
    use crate::untagged_sum::UnionFields;
    use crate::untagged_sum::UntaggedSum;

    pub(super) trait Reducer<Variants: SumVariants, Output>: Product {
        unsafe fn reduce<Fields: UnionFields>(
            self,
            idx: usize,
            untagged_sum: UntaggedSum<Fields>,
        ) -> Output;
    }

    impl<Output> Reducer<Nil, Output> for Nil {
        #[inline]
        unsafe fn reduce<Fields: UnionFields>(
            self,
            _idx: usize,
            _untagged_sum: UntaggedSum<Fields>,
        ) -> Output {
            unreachable!()
        }
    }

    impl<HeadDisc, TailDisc, HeadHandler, TailHandler, Output>
        Reducer<Cons<HeadDisc, TailDisc>, Output> for Cons<HeadHandler, TailHandler>
    where
        TailDisc: SumVariants,
        HeadHandler: FnOnce(HeadDisc) -> Output,
        TailHandler: Reducer<TailDisc, Output>,
    {
        #[inline]
        unsafe fn reduce<Fields: UnionFields>(
            self,
            idx: usize,
            untagged_sum: UntaggedSum<Fields>,
        ) -> Output {
            if idx == 0 {
                let head = unsafe { UntaggedSum::into::<HeadDisc>(untagged_sum) };
                return (self.0)(head);
            }

            unsafe { self.1.reduce(idx - 1, untagged_sum) }
        }
    }

    pub(super) trait ReducerRef<'a, Variants: SumVariants, Output>: Product {
        unsafe fn reduce_ref<Fields: UnionFields>(
            self,
            idx: usize,
            untagged_sum: &'a UntaggedSum<Fields>,
        ) -> Output;
    }

    impl<'a, Output> ReducerRef<'a, Nil, Output> for Nil {
        #[inline]
        unsafe fn reduce_ref<Fields: UnionFields>(
            self,
            _idx: usize,
            _untagged_sum: &'a UntaggedSum<Fields>,
        ) -> Output {
            unreachable!()
        }
    }

    impl<'a, HeadDisc, TailDisc, Head, Tail, Output>
        ReducerRef<'a, Cons<HeadDisc, TailDisc>, Output> for Cons<Head, Tail>
    where
        HeadDisc: 'a,
        TailDisc: SumVariants,
        Head: FnOnce(&'a HeadDisc) -> Output,
        Tail: ReducerRef<'a, TailDisc, Output>,
    {
        #[inline]
        unsafe fn reduce_ref<Fields: UnionFields>(
            self,
            idx: usize,
            untagged_sum: &'a UntaggedSum<Fields>,
        ) -> Output {
            if idx == 0 {
                let head = unsafe { untagged_sum.as_ref::<HeadDisc>() };
                return (self.0)(head);
            }

            unsafe { self.1.reduce_ref(idx - 1, untagged_sum) }
        }
    }
}

/// Product that consists of functions which can be applied to the variants of a [`Sum`]
#[expect(private_bounds, reason = "We want to hide the internal components")]
pub trait Reducer<Variants: SumVariants, Output>:
    internal_reducer::Reducer<Variants, Output>
{
}

impl<Variants, Output, T> Reducer<Variants, Output> for T
where
    Variants: SumVariants,
    T: internal_reducer::Reducer<Variants, Output>,
{
}

/// Like [`ReducerRef`] but borrows the [`Sum`] variant instead
#[expect(private_bounds, reason = "We want to hide the internal components")]
pub trait ReducerRef<'a, Variants: SumVariants, Output>:
    internal_reducer::ReducerRef<'a, Variants, Output>
{
}

impl<'a, Variants, Output, T> ReducerRef<'a, Variants, Output> for T
where
    Variants: SumVariants,
    T: internal_reducer::ReducerRef<'a, Variants, Output>,
{
}
