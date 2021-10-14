#![cfg_attr(not(any(feature = "std", doc)), no_std)]
#![cfg_attr(
    any(feature = "global", doc, all(feature = "std", test)),
    feature(once_cell)
)]
#![feature(
    doc_cfg,
    generic_associated_types,
    ptr_metadata,
    unsize,
    option_result_unwrap_unchecked
)]
#![deny(missing_docs)]

//! rattish enables dynamic casting between different trait objects.
//!
//! This functionality requires runtime type information that isn't
//! automatically created by the Rust compiler and so must be generated
//! manually.
//!
//! rattish is presently only experimental, and depends on unstable compiler
//! features including [`generic_associated_types`], [`ptr_metadata`] and
//! [`unsize`]; [`once_cell`] is used by [`DB`] (enabled by the `global`
//! feature).  Accordingly, a nightly toolchain is required.
//!
//! # Example
//! ```rust
//! #![feature(generic_associated_types, once_cell)]
//! # #[cfg(feature = "global")] {
//!
//! use rattish::{coercible_trait, rtti_global, GlobalDynCast};
//! use std::{any::Any, cell::RefCell, fmt, rc::Rc};
//!
//! // Casting from an object of trait Foo requires Foo to have
//! // super-trait Any..
//! trait Foo: Any {}
//! // ..and that Foo implements both Coercible and InnermostTypeId,
//! // for which there is a macro:
//! coercible_trait!(Foo);
//!
//! // Casting to an object of trait Bar does not require anything
//! // special..
//! trait Bar {
//!     fn double(&self) -> i32;
//! }
//!
//! struct Qux(i32);
//! impl Foo for Qux {}
//! impl Bar for Qux {
//!     fn double(&self) -> i32 {
//!         self.0 * 2
//!     }
//! }
//!
//! fn main() {
//!     // ..except that Bar must be registered in the database with
//!     // each implementing (concrete) type that might underlie a
//!     // dynamic cast to one of its objects
//!     rtti_global! {
//!         Bar: Qux,
//!
//!         // example of another trait with multiple implementations
//!         fmt::LowerExp: f32 i32 u32,
//!     }
//!
//!     // Casting works transitively through any Coercible type.
//!     // Implementations are provided for all standard pointer and
//!     // wrapper types; here, for example, are Rc and RefCell:
//!     let foo: Rc<RefCell<dyn Foo>> = Rc::new(RefCell::new(Qux(123)));
//!
//!     // Explicit type annotation not required; only shown here to
//!     // prove that we actually get an Rc<RefCell<dyn Bar>>
//!     let bar: Rc<RefCell<dyn Bar>>
//!         = foo.dyn_cast::<dyn Bar>().ok().unwrap();
//!
//!     // Lo!  We have indeed casted between trait objects.
//!     assert_eq!(bar.borrow().double(), 246);
//!
//!     // Enjoy that?  Have another, just for fun:
//!     let float: &dyn Any = &876.543f32;
//!     let exp = float.dyn_cast::<dyn fmt::LowerExp>().ok().unwrap();
//!     assert_eq!(format!("{:e}", exp), "8.76543e2");
//! }
//! # main() }
//! ```
//!
//! # Extending rattish to additional pointer/wrapper types
//!
//! You will need to implement [`Coercible`] and [`InnermostTypeId`] for
//! your type; and also [`Pointer`] if your type is a pointer-type (that
//! is, if it is `Sized + Deref`).
//!
//! [`generic_associated_types`]: https://doc.rust-lang.org/nightly/unstable-book/language-features/generic-associated-types.html
//! [`once_cell`]: https://doc.rust-lang.org/nightly/unstable-book/library-features/once-cell.html
//! [`ptr_metadata`]: https://doc.rust-lang.org/nightly/unstable-book/library-features/ptr-metadata.html
//! [`unsize`]: https://doc.rust-lang.org/nightly/unstable-book/library-features/unsize.html

#[cfg(all(any(feature = "alloc", doc), not(feature = "std")))]
extern crate alloc;

pub mod container;
pub mod db;

#[doc(hidden)]
#[cfg(feature = "trace")]
pub use tracing;

use container::{Coerced, Coercible, InnermostTypeId, Metadata, Pointer};
use core::ptr;
use db::{
    error::{CastError, DatabaseEntryError},
    TypeDatabaseEntryExt, TypeDatabaseExt,
};

#[cfg(any(feature = "global", doc))]
use db::{error::DatabaseError, hash_map::DB};

#[cfg(feature = "trace")]
use core::any::type_name;

/// A type whose implementations can be dynamically determined.
pub trait DynImplements<DB>
where
    Self: InnermostTypeId,
    DB: TypeDatabaseExt,
{
    /// Lookup whether `self`'s ultimate concrete type implements `U` in `db`.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(
        Self = type_name::<Self>(),
        U = type_name::<U>(),
    )))]
    fn dyn_implements<U>(&self, db: &DB) -> Result<bool, DatabaseEntryError<U, &Self>>
    where
        U: 'static + ?Sized,
    {
        db.get_db_entry::<U>()?.implements(&self)
    }
}

/// A type that can be dynamically cast.
pub trait DynCast<DB>
where
    Self: Pointer + InnermostTypeId,
    Self::Inner: Coercible,
    DB: TypeDatabaseExt,
{
    /// Cast `self`'s ultimate concrete type to `U`, if registered as an
    /// implementor of `U` in `db`.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all, fields(
        Self = type_name::<Self>(),
        U = type_name::<U>(),
    )))]
    fn dyn_cast<U>(self, db: &DB) -> Result<Self::Coerced<U>, CastError<U, Self>>
    where
        U: 'static + ?Sized,
        Self::Coerced<U>: Sized,
        Coerced<Self::Inner, U>: ptr::Pointee<Metadata = Metadata<U>>,
    {
        match db.get_db_entry() {
            Ok(entry) => entry.cast(self),
            Err(source) => Err(CastError {
                source: source.into(),
                pointer: self,
            }),
        }
    }
}

impl<DB, P: ?Sized> DynImplements<DB> for P
where
    Self: InnermostTypeId,
    DB: TypeDatabaseExt,
{
}

impl<DB, P> DynCast<DB> for P
where
    Self: Pointer + InnermostTypeId,
    Self::Inner: Coercible,
    DB: TypeDatabaseExt,
{
}

#[cfg(any(feature = "global", doc))]
#[doc(cfg(feature = "global"))]
/// A type whose implementations can be dynamically determined using the global
/// [`DB`].
pub trait GlobalDynImplements
where
    Self: InnermostTypeId,
{
    /// Lookup whether `self`'s ultimate concrete type implements `U` in the
    /// global [`DB`].
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    fn dyn_implements<U>(&self) -> Result<bool, DatabaseEntryError<U, &Self>>
    where
        U: 'static + ?Sized,
    {
        let db = DB.get().ok_or(DatabaseError::NotInitialized)?;
        DynImplements::dyn_implements::<U>(self, db)
    }
}

#[cfg(any(feature = "global", doc))]
#[doc(cfg(feature = "global"))]
/// A type that can be dynamically cast using the global [`DB`].
pub trait GlobalDynCast
where
    Self: Pointer + InnermostTypeId,
    Self::Inner: Coercible,
{
    /// Cast `self`'s ultimate concrete type to `U`, if registered as an
    /// implementor of `U` in the global [`DB`].
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    fn dyn_cast<U>(self) -> Result<Self::Coerced<U>, CastError<U, Self>>
    where
        U: 'static + ?Sized,
        Self::Coerced<U>: Sized,
        Coerced<Self::Inner, U>: ptr::Pointee<Metadata = Metadata<U>>,
    {
        match DB.get() {
            Some(db) => DynCast::dyn_cast::<U>(self, db),
            None => Err(CastError {
                source: DatabaseError::NotInitialized.into(),
                pointer: self,
            }),
        }
    }
}

#[cfg(any(feature = "global", doc))]
#[doc(cfg(feature = "global"))]
impl<P> GlobalDynImplements for P where Self: InnermostTypeId {}

#[cfg(any(feature = "global", doc))]
#[doc(cfg(feature = "global"))]
impl<P> GlobalDynCast for P
where
    Self: Pointer + InnermostTypeId,
    Self::Inner: Coercible,
{
}
