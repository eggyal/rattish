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
//! [`unsize`]; [`once_cell`] is used by [`DB`] (enabled by the `static`
//! feature).  Accordingly, a nightly toolchain is required.
//!
//! # Example
//! ```rust
//! #![feature(generic_associated_types, once_cell)]
//! # #[cfg(feature = "static")]
//! # {
//!
//! use rattish::{coercible_trait, rtti_static, GlobalDynCast};
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
//! fn main() {
//!     // ..except that Bar must be registered in the database with
//!     // each implementing (concrete) type that might underlie a
//!     // dynamic cast to one of its objects
//!     rtti_static! {
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
//!     let int = bar.borrow().bar();
//!     assert_eq!(int, 246);
//!
//!     // Enjoy that?  Have another, just for fun:
//!     let float: &dyn Any = &876.543f32;
//!     let exp = float.dyn_cast::<dyn fmt::LowerExp>().unwrap();
//!     assert_eq!(format!("{:e}", exp), "8.76543e2");
//! }
//! # main()
//! # }
//! ```
//!
//! # Extending rattish to additional pointer/wrapper types
//!
//! You will need to implement [`Coercible`] for your type; and also [`Pointer`]
//! if your type is a pointer-type (that is, if it is `Sized + Deref`).
//!
//! [`generic_associated_types`]: https://doc.rust-lang.org/nightly/unstable-book/language-features/generic-associated-types.html
//! [`once_cell`]: https://doc.rust-lang.org/nightly/unstable-book/library-features/once-cell.html
//! [`ptr_metadata`]: https://doc.rust-lang.org/nightly/unstable-book/library-features/ptr-metadata.html
//! [`unsize`]: https://doc.rust-lang.org/nightly/unstable-book/library-features/unsize.html

#[cfg(any(feature = "alloc", doc))]
extern crate alloc;

pub mod container;
pub mod db;

use container::{Coerced, Coercible, InnermostTypeId, Metadata, Pointer};
use core::ptr;
use db::{TypeDatabase, TypeDatabaseEntryExt};

#[cfg(any(feature = "static", doc))]
use db::hash_map::DB;

/// A type whose implementations can be dynamically determined.
pub trait DynImplements<'a, DB>
where
    Self: InnermostTypeId,
    DB: TypeDatabase,
{
    /// Lookup whether `self`'s ultimate concrete type implements `U` in `db`.
    fn dyn_implements<U>(&'a self, db: &DB) -> bool
    where
        U: 'static + ?Sized,
    {
        db.get_entry::<U>()
            .map(|types| types.implements(self))
            .unwrap_or(false)
    }
}

/// A type that can be dynamically cast.
pub trait DynCast<'a, DB>
where
    Self: Pointer<'a> + InnermostTypeId,
    Self::Target: Coercible,
    DB: TypeDatabase,
{
    /// Cast `self`'s ultimate concrete type to `U`, if registered as an
    /// implementor of `U` in `db`.
    fn dyn_cast<U>(self, db: &DB) -> Result<Self::Coerced<U>, Self>
    where
        U: 'static + ?Sized,
        Self::Coerced<U>: Sized,
        Coerced<Self::Target, U>: ptr::Pointee<Metadata = Metadata<U>>,
    {
        match db.get_entry() {
            Some(entry) => entry.cast(self),
            None => Err(self),
        }
    }
}

impl<'a, DB, P: ?Sized> DynImplements<'a, DB> for P
where
    Self: InnermostTypeId,
    DB: TypeDatabase,
{
}

impl<'a, DB, P> DynCast<'a, DB> for P
where
    Self: Pointer<'a> + InnermostTypeId,
    Self::Target: Coercible,
    DB: TypeDatabase,
{
}

#[cfg(any(feature = "static", doc))]
#[doc(cfg(feature = "static"))]
/// A type whose implementations can be dynamically determined using the global
/// [`DB`].
pub trait GlobalDynImplements<'a>
where
    Self: InnermostTypeId,
{
    /// Lookup whether `self`'s ultimate concrete type implements `U` in the
    /// global [`DB`].
    fn dyn_implements<U>(&'a self) -> bool
    where
        U: 'static + ?Sized,
    {
        DynImplements::dyn_implements::<U>(self, DB.get().expect("initialized database"))
    }
}

#[cfg(any(feature = "static", doc))]
#[doc(cfg(feature = "static"))]
/// A type that can be dynamically cast using the global [`DB`].
pub trait GlobalDynCast<'a>
where
    Self: Pointer<'a> + InnermostTypeId,
    Self::Target: Coercible,
{
    /// Cast `self`'s ultimate concrete type to `U`, if registered as an
    /// implementor of `U` in the global [`DB`].
    fn dyn_cast<U>(self) -> Result<Self::Coerced<U>, Self>
    where
        U: 'static + ?Sized,
        Self::Coerced<U>: Sized,
        Coerced<Self::Target, U>: ptr::Pointee<Metadata = Metadata<U>>,
    {
        DynCast::dyn_cast::<U>(self, DB.get().expect("initialized database"))
    }
}

#[cfg(any(feature = "static", doc))]
#[doc(cfg(feature = "static"))]
impl<'a, P> GlobalDynImplements<'a> for P where Self: InnermostTypeId {}

#[cfg(any(feature = "static", doc))]
#[doc(cfg(feature = "static"))]
impl<'a, P> GlobalDynCast<'a> for P
where
    Self: Pointer<'a> + InnermostTypeId,
    Self::Target: Coercible,
{
}
