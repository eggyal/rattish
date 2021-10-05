# RaTTIsh

rattish enables dynamic casting between different trait objects.

This functionality requires runtime type information that isn't
automatically created by the Rust compiler and so must be generated
manually.

rattish is presently only experimental, and depends on unstable compiler
features including [`generic_associated_types`], [`ptr_metadata`] and
[`unsize`]; [`once_cell`] is used by [`DB`] (enabled by the `global`
feature).  Accordingly, a nightly toolchain is required.

## Example
```rust
#![feature(generic_associated_types, once_cell)]

use rattish::{coercible_trait, rtti_global, GlobalDynCast};
use std::{any::Any, cell::RefCell, fmt, rc::Rc};

// Casting from an object of trait Foo requires Foo to have
// super-trait Any..
trait Foo: Any {}
// ..and that Foo implements both Coercible and InnermostTypeId,
// for which there is a macro:
coercible_trait!(Foo);

// Casting to an object of trait Bar does not require anything
// special..
trait Bar {
    fn bar(&self) -> i32;
}

struct Qux(i32);
impl Foo for Qux {}
impl Bar for Qux {
    fn bar(&self) -> i32 {
        self.0 * 2
    }
}

fn main() {
    // ..except that Bar must be registered in the database with
    // each implementing (concrete) type that might underlie a
    // dynamic cast to one of its objects
    rtti_global! {
        Bar: Qux,

        // example of another trait with multiple implementations
        fmt::LowerExp: f32 i32 u32,
    }

    // Casting works transitively through any Coercible type.
    // Implementations are provided for all standard pointer and
    // wrapper types; here, for example, are Rc and RefCell:
    let foo: Rc<RefCell<dyn Foo>> = Rc::new(RefCell::new(Qux(123)));

    // Explicit type annotation not required; only shown here to
    // prove that we actually get an Rc<RefCell<dyn Bar>>
    let bar: Rc<RefCell<dyn Bar>>
        = foo.dyn_cast::<dyn Bar>().ok().unwrap();

    // Lo!  We have indeed casted between trait objects.
    let int = bar.borrow().bar();
    assert_eq!(int, 246);

    // Enjoy that?  Have another, just for fun:
    let float: &dyn Any = &876.543f32;
    let exp = float.dyn_cast::<dyn fmt::LowerExp>().unwrap();
    assert_eq!(format!("{:e}", exp), "8.76543e2");
}
```

## Extending rattish to additional pointer/wrapper types

You will need to implement [`Coercible`] and [`InnermostTypeId`] for
your type; and also [`Pointer`] if your type is a pointer-type (that
is, if it is `Sized + Deref`).

[`generic_associated_types`]: https://doc.rust-lang.org/nightly/unstable-book/language-features/generic-associated-types.html
[`once_cell`]: https://doc.rust-lang.org/nightly/unstable-book/library-features/once-cell.html
[`ptr_metadata`]: https://doc.rust-lang.org/nightly/unstable-book/library-features/ptr-metadata.html
[`unsize`]: https://doc.rust-lang.org/nightly/unstable-book/library-features/unsize.html

[`DB`]: https://docs.rs/rattish/latest/rattish/db/hash_map/static.DB.html
[`Coercible`]: https://docs.rs/rattish/latest/rattish/container/trait.Coercible.html
[`InnermostTypeId`]: https://docs.rs/rattish/latest/rattish/container/trait.InnermostTypeId.html
[`Pointer`]: https://docs.rs/rattish/latest/rattish/container/trait.Pointer.html
