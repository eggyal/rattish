//! Descriptions of types that inherit
//! [`Pointee::Metadata`][ptr::Pointee::Metadata] from a contained type, and how
//! to coerce them.

#[macro_use]
mod macros;
mod impls;

use core::{any::TypeId, ptr};

/// The [`Pointee::Metadata`][ptr::Pointee::Metadata] of `U`.
pub type Metadata<U> = <U as ptr::Pointee>::Metadata;

/// A type that inherits [`Pointee::Metadata`][ptr::Pointee::Metadata]
/// from a contained type, and whose pointers are therefore coercible if that
/// contained type is coercible.  Similar semantics as [`InnermostTypeId`]
/// except also safe for raw pointers.
///
/// # Safety
/// The safety of unsafe code in rattish depends upon this trait being correctly
/// implemented.
pub unsafe trait Coercible {
    /// The type that `Self` will become if the contained type is coerced to
    /// `U`.
    ///
    /// Note that this is a generic associated type, parameters of which are not
    /// presently rendered by Rustdoc.  Its full declaration is:
    ///
    /// ```ignore
    /// type Coerced<U: 'static + ?Sized>: ?Sized
    /// ```
    ///
    /// For example: `&mut T` becomes `&mut U`, `Box<T>` becomes `Box<U>`, etc.
    /// A `dyn Trait` will generally just become `U`.
    type Coerced<U: 'static + ?Sized>: ?Sized;

    /// The ultimate type whose
    /// [`Pointee::Metadata`][ptr::Pointee::Metadata] is that of `Self`.  For
    /// example, if `Self::Metadata` is [`DynMetadata<T>`][ptr::DynMetadata],
    /// then `Innermost` will be `T`.
    ///
    /// Unless `Self` is a leaf, such as a `dyn Trait` (in which case this is
    /// just `Self`), this should just delegate to the contained type's
    /// `Innermost`.
    type Innermost: ?Sized;
}

/// The type that `T` will become if its contained type is coerced to `U`.
#[allow(type_alias_bounds)]
pub type Coerced<T: Coercible, U> = T::Coerced<U>;

/// A type that inherits [`Pointee::Metadata`][ptr::Pointee::Metadata]
/// from a contained type, and whose pointers are therefore coercible if that
/// contained type is coercible.
///
/// # Safety
/// The safety of unsafe code in rattish depends upon this trait being correctly
/// implemented.
pub unsafe trait InnermostTypeId {
    /// Returns the [`TypeId`] of the *concrete* value underlying
    /// [`<Self as ptr::Pointee>::Metadata`].
    ///
    /// Unless `Self` is a leaf, such as a `dyn Trait` (in which case this
    /// should delegate to a trait method), this should just delegate to the
    /// contained type's `innermost_type_id`.
    fn innermost_type_id(&self) -> TypeId;
}

/// A pointer-type to a [`Coercible`].
pub trait Pointer<'a>
where
    Self: Coercible + Sized,
{
    /// Basically the same as [`Deref::Target`][core::ops::Deref::Target], but
    /// without any dereferencing so is safe for raw pointers too.
    type Target: ?Sized + Coercible;

    /// Perform the coercion.
    ///
    /// # Safety
    /// `metadata` must be correct for `Self::Target`.
    unsafe fn coerce<U>(self, metadata: Metadata<Coerced<Self::Target, U>>) -> Self::Coerced<U>
    where
        U: ?Sized,
        Self::Coerced<U>: Sized;
}
