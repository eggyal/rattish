//! Descriptions of types that inherit
//! [`Pointee::Metadata`][ptr::Pointee::Metadata] from a contained type, and how
//! to coerce them.

#[macro_use]
mod macros;
mod impls;

use core::{any::TypeId, ops::Deref, ptr};

type NonNullMut<T> = ptr::NonNull<T>;

/// `*const T` but non-zero.
pub struct NonNullConst<T: ?Sized>(*const T);

impl<T: ?Sized> NonNullConst<T> {
    /// Returns a shared reference to the value.
    ///
    /// # Safety
    ///
    /// When calling this method, you have to ensure that all of the following
    /// are true:
    ///
    /// * The pointer must be properly aligned.
    ///
    /// * It must be "dereferencable" in the sense defined in the documentation
    ///   of the [`core::ptr`][ptr#safety] module.
    ///
    /// * The pointer must point to an initialized instance of `T`.
    ///
    /// * You must enforce Rust's aliasing rules, since the returned lifetime
    ///   `'a` is arbitrarily chosen and does not necessarily reflect the actual
    ///   lifetime of the data. In particular, for the duration of this
    ///   lifetime, the memory the pointer points to must not get mutated
    ///   (except inside `UnsafeCell`).
    ///
    /// This applies even if the result of this method is unused!
    /// (The part about being initialized is not yet fully decided, but until
    /// it is, the only safe approach is to ensure that they are indeed
    /// initialized.)
    pub unsafe fn as_ref<'a>(&self) -> &'a T {
        &*self.0
    }

    /// Performs the same functionality as [`core::ptr::from_raw_parts`], except
    /// that a `NonNullConst` pointer is returned, as opposed to a raw `*const`
    /// pointer.
    ///
    /// See the documentation of [`core::ptr::from_raw_parts`] for more details.
    pub fn from_raw_parts(data_address: NonNullConst<()>, metadata: Metadata<T>) -> Self {
        Self(ptr::from_raw_parts(data_address.0, metadata))
    }

    /// Casts to a pointer of another type.
    pub const fn cast<U>(self) -> NonNullConst<U> {
        NonNullConst(self.0.cast())
    }
}

impl<T: ?Sized> From<&T> for NonNullConst<T> {
    fn from(u: &T) -> Self {
        Self(u)
    }
}

/// The [`Pointee::Metadata`][ptr::Pointee::Metadata] of `U`.
pub type Metadata<U> = <U as ptr::Pointee>::Metadata;

pub use impls::{ConstCoercer, MutCoercer};

/// A type that inherits [`Pointee::Metadata`][ptr::Pointee::Metadata]
/// from a contained type, and whose pointers are therefore coercible if that
/// contained type is coercible.
pub unsafe trait Coercible<'a> {
    /// The type that `Self` will become if the contained type is coerced to
    /// `U`.
    ///
    /// Note that this is a generic associated type, parameters of which are not
    /// presently rendered by Rustdoc.  Its full declaration is:
    ///
    /// ```ignore
    /// type Coerced<U: 'a + ?Sized>: 'a + ?Sized
    /// ```
    type Coerced<U: 'a + ?Sized>: 'a + ?Sized;

    /// The ultimate type whose
    /// [`Pointee::Metadata`][ptr::Pointee::Metadata] is that of `Self`.  For
    /// example, if `Self::Metadata` is [`DynMetadata<T>`][ptr::DynMetadata],
    /// then `Innermost` will be `T`.
    type Innermost: ?Sized;

    /// Returns the [`TypeId`] of the *concrete* value underlying
    /// [`Innermost`][Coercible::Innermost].
    fn innermost_type_id(&self) -> TypeId;
}

/// The type that `X` will become if its contained type is coerced to `Y`.
pub type Coerced<'a, X, Y> = <X as Coercible<'a>>::Coerced<Y>;

/// A coercer of `P`-style [`Pointer`]s.
pub trait PointerCoercer<'a, P>
where
    P: Pointer<'a, Coercer = Self>,
    P::Target: Coercible<'a>,
{
    /// The type that is required by [`P::repoint::<U>`][Pointer::repoint] in
    /// order to create a `P`-style pointer to `U`.
    ///
    /// Note that this is a generic associated type, parameters of which are not
    /// presently rendered by Rustdoc.  Its full declaration is:
    ///
    /// ```ignore
    /// type Raw<U: ?Sized>
    /// ```
    type Raw<U: ?Sized>;

    /// Coerce `pointer`'s contained type to `U`.
    ///
    /// # Safety
    /// `metadata` must be correct for `pointer`.
    unsafe fn coerce<U>(pointer: P, metadata: Metadata<Coerced<'a, P::Target, U>>) -> P::Coerced<U>
    where
        U: ?Sized,
        P::Coerced<U>: Sized;
}

/// The type that is required by [`P::repoint::<U>`][Pointer::repoint] in order
/// to create a `P`-style pointer to `U`.
#[allow(type_alias_bounds)]
pub type PointerRaw<'a, P: Pointer<'a>, U> =
    <P::Coercer as PointerCoercer<'a, P>>::Raw<Coerced<'a, P::Target, U>>;

/// A pointer-type to a [`Coercible`].
pub trait Pointer<'a>
where
    Self: Coercible<'a> + Deref + Sized,
    Self::Target: Coercible<'a>,
{
    /// The [`PointerCoercer`] for this [`Pointer`].
    type Coercer: PointerCoercer<'a, Self>;

    /// Replace `self` with a `Self`-style pointer to `U`, created from `ptr`.
    ///
    /// # Safety
    /// `ptr` must be valid.
    unsafe fn repoint<U>(self, ptr: PointerRaw<'a, Self, U>) -> Self::Coerced<U>
    where
        U: ?Sized,
        Self::Coerced<U>: Sized;
}
