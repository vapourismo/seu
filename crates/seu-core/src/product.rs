use core::mem;

/// Product type
pub trait Product {
    const LEN: usize;
}

/// Empty product
pub struct Nil;

impl Product for Nil {
    const LEN: usize = 0;
}

/// Prepend a head item onto a product tail
pub struct Cons<Head, Tail: Product>(pub Head, pub Tail);

impl<Head, Tail: Product> Cons<Head, Tail> {
    pub const HEAD_OFFSET: usize = mem::offset_of!(Self, 0);

    pub const TAIL_OFFSET: usize = mem::offset_of!(Self, 1);
}

impl<Head, Tail: Product> Product for Cons<Head, Tail> {
    const LEN: usize = 1 + Tail::LEN;
}

/// Construct a product expression from a list of expressions.
///
/// # Example
///
/// ```
/// use seu_core::product;
///
/// let p = product![1, "hello", true];
/// ```
#[macro_export]
macro_rules! product {
    [] => {
        $crate::product::Nil
    };

    [$head:expr $(, $tail:expr)* $(,)?] => {
        $crate::product::Cons($head, $crate::product![$($tail),*])
    };
}

/// Construct a product pattern from a list of patterns.
///
/// # Example
///
/// ```
/// use seu_core::{product, product_pat};
///
/// let product_pat![x, y, z] = product![1, "hello", true];
/// ```
#[macro_export]
macro_rules! product_pat {
    [] => {
        $crate::product::Nil
    };

    [$head:pat $(, $tail:pat)* $(,)?] => {
        $crate::product::Cons($head, $crate::product_pat![$($tail),*])
    };
}

/// Construct a product type from a list of types.
///
/// # Example
///
/// ```
/// use seu_core::{product, Product};
///
/// let p: Product![i32, bool, String] = product![1, true, "hello".to_string()];
/// ```
#[macro_export]
macro_rules! Product {
    [] => {
        $crate::product::Nil
    };

    [$head:ty $(, $tail:ty)* $(,)?] => {
        $crate::product::Cons<$head, $crate::Product![$($tail),*]>
    };
}
