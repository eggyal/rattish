//! Descriptions of types that inherit
//! [`Pointee::Metadata`][ptr::Pointee::Metadata] from a contained type, and how
//! to coerce them.

#[macro_use]
mod macros;
mod impls;

use core::{any::TypeId, ops::Deref, ptr};

/// The [`Pointee::Metadata`][ptr::Pointee::Metadata] of `U`.
pub type Metadata<U> = <U as ptr::Pointee>::Metadata;

/// A type that inherits [`Pointee::Metadata`][ptr::Pointee::Metadata]
/// from a contained type, and whose pointers are therefore coercible if that
/// contained type is coercible.
pub unsafe trait Coercible {
    /// The type that `Self` will become if the contained type is coerced to
    /// `U`.
    ///
    /// Note that this is a generic associated type, parameters of which are not
    /// presently rendered by Rustdoc.  Its full declaration is:
    ///
    /// ```ignore
    /// type Coerced<U: ?Sized>: ?Sized
    /// ```
    type Coerced<'a, U: 'a + ?Sized>: 'a + ?Sized;

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
#[allow(type_alias_bounds)]
pub type Coerced<'a, X: Coercible, Y> = X::Coerced<'a, Y>;

/// Reference coercion.
pub trait Coerce {
    /// Perform the coercion.
    ///
    /// # Safety
    /// `metadata` must be correct for `Self`.
    unsafe fn coerce_ref<U>(&self, metadata: Metadata<U>) -> &U
    where
        U: ?Sized;

    /// Perform the coercion.
    ///
    /// # Safety
    /// `metadata` must be correct for `Self`.
    unsafe fn coerce_mut<U>(&mut self, metadata: Metadata<U>) -> &mut U
    where
        U: ?Sized;
}

/// A pointer-type to a [`Coercible`].
pub trait Pointer<'a>
where
    Self: Coercible + Deref + Sized,
    Self::Target: Coercible,
{
    /// Perform the coercion.
    ///
    /// # Safety
    /// `metadata` must be correct for `Self::Target`.
    unsafe fn coerce<U>(
        self,
        metadata: Metadata<Coerced<'a, Self::Target, U>>,
    ) -> Self::Coerced<'a, U>
    where
        U: ?Sized,
        Self::Coerced<'a, U>: Sized;
    // Coerced<'a, Self::Innermost, U>: ptr::Pointee<Metadata = Metadata<U>>;
}
