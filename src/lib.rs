#![cfg_attr(not(any(feature = "std", doc)), no_std)]
#![cfg_attr(any(feature = "static", doc), feature(once_cell))]
#![feature(doc_cfg, generic_associated_types, ptr_metadata, unsize)]
#![deny(missing_docs)]

//! rattish enables dynamic casting between different trait objects.
//!
//! This functionality requires runtime type information that isn't
//! automatically created by the Rust compiler and so must be generated
//! manually.
//!
//! rattish is presently only experimental, and depends on unstable compiler
//! features including [`generic_associated_types`], [`ptr_metadata`] and
//! [`unsize`].  Accordingly, a nightly toolchain is required.
//!
//! # Example
//! ```rust
//! # #![feature(generic_associated_types, once_cell)]
//! # #[cfg(feature = "static")]
//! # {
//! use rattish::{coercible_trait, rtti_static, StaticDynCast};
//! use std::{any::Any, cell::RefCell, fmt, rc::Rc};
//!
//! // casting from an object of trait Foo requires Foo to have
//! // super-trait Any..
//! trait Foo: Any {}
//! // ..and that Foo implements Coercible, for which there is a macro
//! coercible_trait!(Foo);
//!
//! // casting to an object of trait Bar does not require anything
//! // special..
//! trait Bar {
//!     fn bar(&self) -> i32;
//! }
//!
//! struct Qux(i32);
//! impl Foo for Qux {}
//! impl Bar for Qux {
//!     fn bar(&self) -> i32 {
//!         self.0 * 2
//!     }
//! }
//!
//! // ..except that Bar must be registered in the database with every
//! // one of its concrete types that might get dynamically cast to a
//! // Bar trait object
//! rtti_static! {
//!     Bar: Qux,
//!
//!     // example of another trait with multiple implementations
//!     fmt::LowerExp: u32 i32 f32,
//! }
//!
//! // casting works through any type that implements Coercible
//! // implementations are provided for all standard pointer and wrapper
//! // types; here, for example, are Rc and RefCell:
//! let foo: Rc<RefCell<dyn Foo>> = Rc::new(RefCell::new(Qux(123)));
//!
//! // explicit type annotation not required; only included here for
//! // information
//! let bar: Rc<RefCell<dyn Bar>> = foo.dyn_cast::<dyn Bar>().ok().unwrap();
//! let int = bar.borrow().bar();
//! assert_eq!(int, 246);
//!
//! // enjoyed that?  have another, just for fun
//! let exp = (&int as &dyn Any).dyn_cast::<dyn fmt::LowerExp>().unwrap();
//! assert_eq!(format!("{:e}", exp), "2.46e2");
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

#[cfg(any(feature = "static", doc))]
use db::hash_map::DB;

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
pub trait DynCast<'a, DB>
where
    Self: Pointer<'a>,
    Self::Target: Coercible,
    DB: TypeDatabase,
{
    /// Downcast `self`'s ultimate concrete type to `U`, if registered as an
    /// implementor of `U` in `db`.
    #[inline(always)]
    fn dyn_cast<U>(self, db: &DB) -> Result<Self::Coerced<'a, U>, Self>
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

impl<'a, DB, P> DynCast<'a, DB> for P
where
    Self: Pointer<'a>,
    Self::Target: Coercible,
    DB: TypeDatabase,
{
}

#[cfg(any(feature = "static", doc))]
#[doc(cfg(feature = "static"))]
/// A type whose implementations can be dynamically determined using [`DB`].
pub trait StaticDynImplements<'a>
where
    Self: Coercible,
{
    /// Lookup whether `self`'s ultimate concrete type implements `U` in [`DB`].
    #[inline(always)]
    fn dyn_implements<U>(&'a self) -> bool
    where
        U: 'static + ?Sized,
    {
        DynImplements::dyn_implements::<U>(&self, DB.get().expect("initialized database"))
    }
}

#[cfg(any(feature = "static", doc))]
#[doc(cfg(feature = "static"))]
/// A type that can be dynamically downcast using [`DB`].
pub trait StaticDynCast<'a>
where
    Self: Pointer<'a>,
    Self::Target: Coercible,
{
    /// Downcast `self`'s ultimate concrete type to `U`, if registered as an
    /// implementor of `U` in [`DB`].
    #[inline(always)]
    fn dyn_cast<U>(self) -> Result<Self::Coerced<'a, U>, Self>
    where
        U: 'static + ?Sized,
        Self::Coerced<'a, U>: Sized,
        Coerced<'a, Self::Target, U>: ptr::Pointee<Metadata = Metadata<U>>,
    {
        DynCast::dyn_cast::<U>(self, DB.get().expect("initialized database"))
    }
}

#[cfg(any(feature = "static", doc))]
#[doc(cfg(feature = "static"))]
impl<'a, P> StaticDynImplements<'a> for P where Self: Coercible {}

#[cfg(any(feature = "static", doc))]
#[doc(cfg(feature = "static"))]
impl<'a, P> StaticDynCast<'a> for P
where
    Self: Pointer<'a>,
    Self::Target: Coercible,
{
}
