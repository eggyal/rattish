//! Traits that describe types that inherit
//! [`Pointee::Metadata`][ptr::Pointee::Metadata] from a contained type, and how
//! to coerce them.
//!
//! All cases should implement [`Coercible`]; anything that can dereference its
//! contained type (which should be every [`Coercible`] except raw pointers)
//! should also implement [`InnermostTypeId`]; and anything that is a
//! pointer-type (which should be every [`Coercible`] that is [`Sized`]) should
//! also implement [`Pointer`].
//!
//! Implementations are provided for standard library types.

#[macro_use]
mod macros;
mod impls;

use core::{any::TypeId, ptr};

/// The [`Pointee::Metadata`][ptr::Pointee::Metadata] of `U`.
pub type Metadata<U> = <U as ptr::Pointee>::Metadata;

/// A type that inherits [`Pointee::Metadata`][ptr::Pointee::Metadata]
/// from a contained type, pointers to which are therefore coercible if that
/// contained type is coercible.
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
    /// For example: `&mut T` becomes `&mut T::Coerced<U>`, `Box<T>` becomes
    /// `Box<T::Coerced<U>>`, etc.  A `dyn Trait` will generally just become
    /// `U`.
    type Coerced<U: 'static + ?Sized>: ?Sized;

    /// The directly contained type, similar to
    /// [`Deref::Target`][core::ops::Deref::Target] but without any
    /// dereferencing (so is safe for raw pointers too).  For example: `T` is
    /// the inner type of an `&mut T`, a `Box<T>`, etc.  The inner type of a
    /// leaf such as `dyn Trait` is `Self`.
    type Inner: ?Sized + Coercible;

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

/// Error that arose whilst determining a pointee's concrete type.
#[derive(Debug)]
#[cfg_attr(feature = "thiserror", derive(thiserror::Error))]
pub enum TypeIdDeterminationError {
    /// The concrete type could not be determined because the pointer traverses
    /// a weak reference to some data that is no longer available.
    #[cfg_attr(feature = "thiserror", error("unable to upgrade {type_name}"))]
    UnableToUpgradeWeakReference {
        /// The name of the Weak reference type that could not be upgraded
        type_name: &'static str,
    },
}

/// A dereferenceable type that inherits
/// [`Pointee::Metadata`][ptr::Pointee::Metadata] from a contained type,
/// pointers to which are therefore coercible if that contained type is
/// coercible.
///
/// # Safety
/// The safety of unsafe code in rattish depends upon this trait being correctly
/// implemented.
pub unsafe trait InnermostTypeId {
    /// Returns the [`TypeId`] of the *concrete* value underlying
    /// [`<Self as ptr::Pointee>::Metadata`][ptr::Pointee::Metadata].
    ///
    /// Unless `Self` is a leaf, such as a `dyn Trait` (in which case this
    /// should delegate to a trait method like
    /// [`Any::type_id`][core::any::Any::type_id]), this should just delegate to
    /// the contained type's `innermost_type_id`.
    fn innermost_type_id(&self) -> Result<TypeId, TypeIdDeterminationError>;
}

/// A [`Sized`] type that inherits [`Pointee::Metadata`][ptr::Pointee::Metadata]
/// from a contained type, and therefore is a "pointer" to that type; as such,
/// it is coercible if that contained type is coercible.
pub trait Pointer
where
    Self: Coercible + Sized,
{
    /// Perform the coercion.  In most cases this will utilise the [`Pointer`]
    /// implementation for some more primitive type, e.g. a reference or a raw
    /// pointer.
    ///
    /// # Safety
    /// `metadata` must be correct for `Self::Inner`.
    unsafe fn coerce<U>(self, metadata: Metadata<Coerced<Self::Inner, U>>) -> Self::Coerced<U>
    where
        U: ?Sized,
        Self::Coerced<U>: Sized;
}
