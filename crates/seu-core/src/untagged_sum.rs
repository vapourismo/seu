use core::mem;
use core::mem::ManuallyDrop;
use core::mem::MaybeUninit;

use crate::nums::Index;

mod internal {
    use core::mem::ManuallyDrop;

    use crate::product::Cons;
    use crate::product::Nil;
    use crate::product::Product;

    pub(super) trait UnionFields: Product {
        type UnionRepr;
    }

    pub(super) struct NilRepr;

    impl UnionFields for Nil {
        type UnionRepr = NilRepr;
    }

    pub(super) union ConsRepr<Head, Tail: UnionFields> {
        _head: ManuallyDrop<Head>,
        _tail: ManuallyDrop<Tail::UnionRepr>,
    }

    impl<Head, Tail: UnionFields> UnionFields for Cons<Head, Tail> {
        type UnionRepr = ConsRepr<Head, Tail>;
    }
}

/// Marks products as eligible to be used as fields in an untagged sum.
#[expect(private_bounds, reason = "We want to hide the internal components")]
pub trait UnionFields: internal::UnionFields {}

impl<T: internal::UnionFields> UnionFields for T {}

/// Untagged union type
///
/// This is like a `union` in Rust.
///
/// # Drop behaviour
///
/// You must explicitly drop the value using `drop_as`. By default, the value is not dropped when
/// the untagged sum goes out of scope.
#[repr(transparent)]
pub struct UntaggedSum<Fields: UnionFields> {
    repr: Fields::UnionRepr,
}

impl<Fields: UnionFields> UntaggedSum<Fields> {
    /// Construct an untagged union using a value of one of the fields.
    pub fn new<Idx: Index<Fields>>(_index: Idx, data: ManuallyDrop<Idx::Selected>) -> Self {
        debug_assert!(size_of::<Fields::UnionRepr>() >= size_of::<Idx::Selected>());

        // SAFETY: The fields representation is at least as large as `data` and properly aligned
        // for us to write to the space.
        let repr: Fields::UnionRepr = unsafe {
            let mut inner: MaybeUninit<Fields::UnionRepr> = MaybeUninit::uninit();
            inner
                .as_mut_ptr()
                .cast::<ManuallyDrop<Idx::Selected>>()
                .write(data);
            inner.assume_init()
        };

        Self { repr }
    }

    /// Get a reference to the value stored in this untagged sum.
    ///
    /// # Safety
    ///
    /// The caller must know that the value currently stored in this sum is of type `T`.
    pub unsafe fn as_ref<T>(&self) -> &T {
        unsafe { mem::transmute::<&<Fields as internal::UnionFields>::UnionRepr, &T>(&self.repr) }
    }

    /// Get a mutable reference to the value stored in this untagged sum.
    ///
    /// # Safety
    ///
    /// The caller must know that the value currently stored in this sum is of type `T`.
    pub unsafe fn as_mut<T>(&mut self) -> &mut T {
        unsafe {
            mem::transmute::<&mut <Fields as internal::UnionFields>::UnionRepr, &mut T>(
                &mut self.repr,
            )
        }
    }

    /// Get a raw pointer to the value stored in this untagged sum.
    pub fn as_ptr(&self) -> *const () {
        &self.repr as *const <Fields as internal::UnionFields>::UnionRepr as *const ()
    }

    /// Convert the untagged sum into a value of type `T`, consuming the sum.
    ///
    /// # Safety
    ///
    /// The caller must know that the value currently stored in this sum is of type `T`.
    pub unsafe fn into<T>(self) -> T {
        unsafe {
            let value = self.as_ptr().cast::<T>().read();
            mem::forget(self);
            value
        }
    }

    /// Drop the value stored in this untagged sum as type `T`.
    ///
    /// # Safety
    ///
    /// The caller must know that the value currently stored in this sum is of type `T`.
    pub unsafe fn drop_as<T>(&mut self) {
        unsafe {
            let value =
                mem::transmute::<&mut Fields::UnionRepr, &mut ManuallyDrop<T>>(&mut self.repr);
            ManuallyDrop::drop(value);
        }
    }
}
