//! A database for runtime type information.

pub mod error;

#[cfg(all(test, feature = "std"))]
mod tests;

#[cfg(any(feature = "std", doc))]
#[doc(cfg(feature = "std"))]
pub mod hash_map;

use crate::container::{Coerced, Coercible, InnermostTypeId, Metadata, Pointer};
use core::{
    any::TypeId,
    marker::{PhantomData, Unsize},
    ptr,
};
use error::{CastError, DatabaseEntryError, DatabaseError};

#[cfg(feature = "trace")]
use core::any::type_name;

/// A key-value store, where the key is the [`TypeId`] of a concrete Rust type
/// and the value is that type's [`Metadata<U>`].
///
/// `U` will typically be `dyn Trait` such that the value stored for a given
/// type's `TypeId` is its vtable for `Trait`.
///
/// # Safety
/// [`metadata`][TypeDatabaseEntry::metadata] must only ever return `Some(&m)`
/// if `m` was previously [`add`][TypeDatabaseEntry::add]ed for the given
/// `type_id`.
pub unsafe trait TypeDatabaseEntry<U>
where
    U: ?Sized,
{
    /// Add `metadata` for the given `type_id`.
    ///
    /// # Safety
    /// `metadata` must be the correct [`Metadata<U>`] for the concrete type
    /// represented by `type_id`.
    unsafe fn add(&mut self, type_id: TypeId, metadata: Metadata<U>);

    /// Whether this store contains metadata for `type_id`.
    fn contains<'a>(&self, type_id: TypeId) -> bool;

    /// A reference to the metadata, if any, previously
    /// [`add`][TypeDatabaseEntry::add]ed for the given `type_id`.
    fn metadata(&self, type_id: TypeId) -> Option<&Metadata<U>>;
}

/// The consumer interface for a [`TypeDatabaseEntry<U>`].
pub trait TypeDatabaseEntryExt<U>
where
    Self: TypeDatabaseEntry<U>,
    U: ?Sized,
{
    /// Register concrete type `I` as an implementor of `U`.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(
        U = type_name::<U>(),
        I = type_name::<I>(),
    )))]
    fn register<I>(&mut self)
    where
        I: 'static + Unsize<U>,
    {
        unsafe {
            let type_id = TypeId::of::<I>();
            let metadata = ptr::metadata::<U>(ptr::null::<I>() as _);
            self.add(type_id, metadata);
        }
    }

    /// Attempt to determine the concrete type of the given `data`.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    fn concrete_type_id<'a, P>(&self, data: &P) -> Result<TypeId, DatabaseEntryError<U, P>>
    where
        P: ?Sized + InnermostTypeId,
    {
        Ok(data.innermost_type_id()?)
    }

    /// Whether `data` is registered as an implementor of `U`.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(
        P = type_name::<P>(),
        U = type_name::<U>(),
    )))]
    fn implements<'a, P>(&self, data: &P) -> Result<bool, DatabaseEntryError<U, P>>
    where
        P: ?Sized + InnermostTypeId,
    {
        self.concrete_type_id(data)
            .map(|type_id| self.contains(type_id))
    }

    /// Cast `pointer` to `P::Coerced<U>`, if registered as an implementor of
    /// `U`.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(
        P = type_name::<P>(),
        U = type_name::<U>(),
    )))]
    fn cast<'a, P>(&self, pointer: P) -> Result<P::Coerced<U>, CastError<U, P>>
    where
        P: Pointer + InnermostTypeId,
        P::Coerced<U>: Sized,
        P::Inner: Coercible,
        Coerced<P::Inner, U>: ptr::Pointee<Metadata = Metadata<U>>,
    {
        unsafe {
            match self.concrete_type_id(&pointer).and_then(|type_id| {
                self.metadata(type_id).ok_or(
                    DatabaseEntryError::ConcreteTypeNotRegisteredForTarget {
                        type_id,
                        requested_type: PhantomData,
                        instance_type: PhantomData,
                    },
                )
            }) {
                Ok(&metadata) => Ok(pointer.coerce(metadata)),
                Err(source) => Err(CastError { source, pointer }),
            }
        }
    }
}

impl<U, E> TypeDatabaseEntryExt<U> for E
where
    Self: TypeDatabaseEntry<U>,
    U: ?Sized,
{
}

/// A key-value store, where the key is a Rust type and the value is a
/// [`TypeDatabaseEntry`] parameterized by that key.
///
/// Databases of this type will typically be keyed by `dyn Trait`s and hold
/// values representing the types that implement that `Trait`.
///
/// # Safety
/// Lookups for a given key must always return references to the same value.
pub unsafe trait TypeDatabase {
    /// The type of values keyed by type `U`.
    ///
    /// Note that this is a generic associated type, parameters of which are not
    /// presently rendered by Rustdoc.  Its full declaration is:
    ///
    /// ```ignore
    /// type Entry<U: ?Sized>: TypeDatabaseEntry<U>
    /// ```
    type Entry<U: ?Sized>: TypeDatabaseEntry<U>;

    /// Returns an exclusive/mutable reference to the value of the entry that is
    /// keyed by `U`.  A new entry may be created if one did not previously
    /// exist.
    fn get_entry_mut<U>(&mut self) -> &mut Self::Entry<U>
    where
        U: 'static + ?Sized;

    /// Returns a shared/immutable reference to the value of the entry that is
    /// keyed by `U`.
    fn get_entry<U>(&self) -> Option<&Self::Entry<U>>
    where
        U: 'static + ?Sized;
}

/// The consumer interface of a `TypeDatabase`.
pub trait TypeDatabaseExt
where
    Self: TypeDatabase,
{
    /// Returns a shared/immutable reference to the value of the entry that is
    /// keyed by `U`.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(
        U = type_name::<U>(),
    )))]
    fn get_db_entry<U>(&self) -> Result<&Self::Entry<U>, DatabaseError<U>>
    where
        U: 'static + ?Sized,
    {
        self.get_entry()
            .ok_or(DatabaseError::RequestedTypeNotInDatabase {
                requested_type: PhantomData,
            })
    }
}

impl<DB> TypeDatabaseExt for DB where Self: TypeDatabase {}
