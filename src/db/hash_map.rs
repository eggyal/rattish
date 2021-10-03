//! A [`HashMap`] implementation of a [`TypeDatabase`].

use super::{Metadata, TypeDatabase, TypeDatabaseEntry};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

/// A [`TypeDatabase`] backed by a [`HashMap`].
#[derive(Default)]
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
    #[inline(always)]
    fn default() -> Self {
        HashMapTypeDatabaseEntry(Default::default())
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

#[doc(cfg(feature = "std"))]
unsafe impl<U> TypeDatabaseEntry<U> for HashMapTypeDatabaseEntry<U>
where
    U: ?Sized,
{
    #[inline(always)]
    unsafe fn add(&mut self, type_id: TypeId, metadata: Metadata<U>) {
        self.0.insert(type_id, metadata);
    }

    #[inline(always)]
    fn contains<'a>(&self, type_id: TypeId) -> bool {
        self.0.contains_key(&type_id)
    }

    #[inline(always)]
    fn metadata(&self, type_id: TypeId) -> Option<&Metadata<U>> {
        self.0.get(&type_id)
    }
}

#[doc(cfg(feature = "std"))]
unsafe impl TypeDatabase for HashMapTypeDatabase {
    type Entry<U: ?Sized> = HashMapTypeDatabaseEntry<U>;

    fn get_entry_mut<U>(&mut self) -> &mut Self::Entry<U>
    where
        U: 'static + ?Sized,
    {
        self.0
            .entry(TypeId::of::<U>())
            .or_insert_with(|| Box::new(Self::Entry::<U>::default()))
            .downcast_mut()
            .unwrap()
    }

    #[inline(always)]
    fn get_entry<U>(&self) -> Option<&Self::Entry<U>>
    where
        U: 'static + ?Sized,
    {
        self.0
            .get(&TypeId::of::<U>())
            .and_then(|t| t.downcast_ref())
    }
}