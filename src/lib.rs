#![cfg_attr(not(any(feature = "std", doc)), no_std)]
#![feature(doc_cfg, ptr_metadata, unsize, generic_associated_types)]
#![deny(missing_docs)]

//! Objects of the core library's [`Any`][core::any::Any] trait, of which there
//! is a blanket implementation for all `'static` types (whether [`Sized`] or
//! not), can be downcast to references of their true concrete type: but they
//! cannot be directly downcast to objects of other traits that are implemented
//! by that concrete type.  For example, an `&dyn Any` can be downcast to an
//! `&i32` (if it is indeed an `i32` beneath) but it cannot be directly downcast
//! to, say, an `&dyn Display` (if its underlying concrete type does indeed
//! implement `Display`).
//!
//! This isn't a problem if one *knows* the underlying concrete type, as clearly
//! one can still reach any such implemented trait by first downcasting to that
//! concrete type.  However, if the concrete type is not known, Rust does not
//! have sufficient runtime information available to reach any other implemented
//! traits.
//!
//! [`rattish`][self] provides such a runtime database together with facilities
//! for directly downcasting from an `Any` trait object to objects of any trait
//! that is registered in its database for the underlying concrete type.
//!
//! `rattish` is presently only experimental, and depends on unstable compiler
//! features including [`ptr_metadata`] and [`unsize`] (it also uses
//! [`generic_associated_types`] for some functionality; and, while the code
//! could be refactored to remove that dependency, that feature is close to
//! stabilization).  Accordingly, a nightly toolchain is required.
//!
//! # Example
//! ```rust
//! # #[cfg(feature = "std")]
//! # {
//! use rattish::{rtti, DynDowncast};
//! use std::{any::Any, cell::RefCell, fmt};
//!
//! let db = rtti! {
//!     fmt::LowerExp: u32 f32,
//! };
//!
//! let any_box = Box::new(RefCell::new(123.456f32)) as Box<RefCell<dyn Any>>;
//! let lowerexp_box = any_box.dyn_downcast::<dyn fmt::LowerExp>(&db).unwrap();
//! assert!((&lowerexp_box as &dyn Any).is::<Box<RefCell<dyn fmt::LowerExp>>>());
//! assert_eq!(format!("{:e}", &*RefCell::borrow(&lowerexp_box)), "1.23456e2");
//! # }
//! ```
//!
//! [`generic_associated_types`]: https://doc.rust-lang.org/nightly/unstable-book/language-features/generic-associated-types.html
//! [`ptr_metadata`]: https://doc.rust-lang.org/nightly/unstable-book/library-features/ptr-metadata.html
//! [`unsize`]: https://doc.rust-lang.org/nightly/unstable-book/library-features/unsize.html

#[cfg(any(feature = "alloc", doc))]
extern crate alloc;

pub mod container;
pub mod db;

use container::{Coerced, Coercible, Metadata, Pointer};
use core::ptr;
use db::{TypeDatabase, TypeDatabaseEntryExt};

/// A type whose implementations can be dynamically determined.
pub trait DynImplements<'a, DB>
where
    Self: Coercible,
    DB: TypeDatabase,
{
    /// Lookup whether `self`'s ultimate concrete type implements `U` in `db`.
    #[inline(always)]
    fn dyn_implements<U>(&'a self, db: &DB) -> bool
    where
        U: 'static + ?Sized,
    {
        db.get_entry::<U>()
            .map(|types| types.implements(self))
            .unwrap_or(false)
    }
}

/// A type that can be dynamically downcast.
pub trait DynDowncast<'a, DB>
where
    Self: Pointer<'a>,
    Self::Target: Coercible,
    DB: TypeDatabase,
{
    /// Downcast `self`'s ultimate concrete type to `U`, if registered as an
    /// implementor of `U` in `db`.
    #[inline(always)]
    fn dyn_downcast<U>(self, db: &DB) -> Result<Self::Coerced<'a, U>, Self>
    where
        U: 'static + ?Sized,
        Self::Coerced<'a, U>: Sized,
        Coerced<'a, Self::Target, U>: ptr::Pointee<Metadata = Metadata<U>>,
    {
        match db.get_entry() {
            Some(entry) => entry.downcast(self),
            None => Err(self),
        }
    }
}

impl<'a, DB, P> DynImplements<'a, DB> for P
where
    Self: Coercible,
    DB: TypeDatabase,
{
}

impl<'a, DB, P> DynDowncast<'a, DB> for P
where
    Self: Pointer<'a>,
    Self::Target: Coercible,
    DB: TypeDatabase,
{
}
