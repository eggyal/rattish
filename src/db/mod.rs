//! A database for runtime type information.

#[cfg(any(feature = "std", doc))]
#[doc(cfg(feature = "std"))]
pub mod hash_map;

use crate::container::{Coerced, Coercible, InnermostTypeId, Metadata, Pointer};
use core::{any::TypeId, marker::Unsize, ptr};

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

    /// Whether `data` is registered as an implementor of `U`.
    fn implements<'a, P>(&self, data: P) -> bool
    where
        P: InnermostTypeId,
    {
        match data.innermost_type_id() {
            Some(type_id) => self.contains(type_id),
            None => false,
        }
    }

    /// Cast `pointer` to `P::Coerced<U>`, if registered as an implementor of
    /// `U`.
    fn cast<'a, P>(&self, pointer: P) -> Result<P::Coerced<U>, P>
    where
        P: Pointer + InnermostTypeId,
        P::Coerced<U>: Sized,
        P::Inner: Coercible,
        Coerced<P::Inner, U>: ptr::Pointee<Metadata = Metadata<U>>,
    {
        unsafe {
            match pointer
                .innermost_type_id()
                .and_then(|type_id| self.metadata(type_id))
            {
                Some(&metadata) => Ok(pointer.coerce(metadata)),
                None => Err(pointer),
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
