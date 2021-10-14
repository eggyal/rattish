//! A [`HashMap`] implementation of a [`TypeDatabase`].

use super::{Metadata, TypeDatabase, TypeDatabaseEntry};
use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
    fmt,
};

#[cfg(any(feature = "global", doc))]
use std::lazy::SyncOnceCell;

/// A [`TypeDatabase`] backed by a [`HashMap`].
#[derive(Debug, Default)]
#[doc(cfg(feature = "std"))]
pub struct HashMapTypeDatabase(HashMap<TypeId, Box<dyn Any + Send + Sync>>);

/// A [`TypeDatabaseEntry`] backed by a [`HashMap`].
#[doc(cfg(feature = "std"))]
pub struct HashMapTypeDatabaseEntry<U>(HashMap<TypeId, Metadata<U>>)
where
    U: ?Sized;

impl<U> Default for HashMapTypeDatabaseEntry<U>
where
    U: ?Sized,
{
    fn default() -> Self {
        Self(HashMap::default())
    }
}

impl<U> fmt::Debug for HashMapTypeDatabaseEntry<U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HashMapTypeDatabaseEntry<{}> ", type_name::<U>())?;
        f.debug_set().entries(self.0.keys()).finish()
    }
}

/// Evaluates to a newly instantiated [`HashMapTypeDatabase`], initialized with
/// the provided entries.
#[macro_export]
#[doc(cfg(feature = "std"))]
macro_rules! rtti {
    ($( $trait:path: $( $ty:ty )+, )+) => {{
        use $crate::db::{TypeDatabase, TypeDatabaseEntryExt};
        let mut db = $crate::db::hash_map::HashMapTypeDatabase::default();
        $(
            let entry = db.get_entry_mut::<dyn $trait>();
            $(entry.register::<$ty>();)+
        )+
        db
    }};
}

/// A global, immutable, thread-safe [`HashMapTypeDatabase`] that can be
/// initialized with [`rtti_global`].
#[cfg(any(feature = "global", doc))]
#[doc(cfg(feature = "global"))]
pub static DB: SyncOnceCell<HashMapTypeDatabase> = SyncOnceCell::new();

/// Instantiates the global [`DB`] with the provided entries.
#[macro_export]
#[cfg(any(feature = "global", doc))]
#[doc(cfg(feature = "global"))]
macro_rules! rtti_global {
    ($( $token:tt )+) => {{
        $crate::db::hash_map::DB
            .set($crate::rtti!($($token)+))
            .ok()
            .expect("database already initialized");
    }};
}

unsafe impl<U> TypeDatabaseEntry<U> for HashMapTypeDatabaseEntry<U>
where
    U: ?Sized,
{
    #[cfg_attr(feature = "trace", tracing::instrument(skip(self, metadata)))]
    unsafe fn add(&mut self, type_id: TypeId, metadata: Metadata<U>) {
        let _ = self.0.insert(type_id, metadata);
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip(self)))]
    fn contains(&self, type_id: TypeId) -> bool {
        self.0.contains_key(&type_id)
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip(self)))]
    fn metadata(&self, type_id: TypeId) -> Option<&Metadata<U>> {
        self.0.get(&type_id)
    }
}

unsafe impl TypeDatabase for HashMapTypeDatabase {
    type Entry<U: ?Sized> = HashMapTypeDatabaseEntry<U>;

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(
        U = type_name::<U>(),
    )))]
    fn get_entry_mut<U>(&mut self) -> &mut Self::Entry<U>
    where
        U: 'static + ?Sized,
    {
        unsafe {
            self.0
                .entry(TypeId::of::<U>())
                .or_insert_with(|| Box::new(Self::Entry::<U>::default()))
                .downcast_mut()
                .unwrap_unchecked()
        }
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(
        U = type_name::<U>(),
    )))]
    fn get_entry<U>(&self) -> Option<&Self::Entry<U>>
    where
        U: 'static + ?Sized,
    {
        self.0
            .get(&TypeId::of::<U>())
            .and_then(|t| t.downcast_ref())
    }
}
