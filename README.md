# RaTTIsh

rattish enables dynamic casting between different trait objects.

This functionality requires runtime type information that isn't
automatically created by the Rust compiler and so must be generated
manually.

rattish is presently only experimental, and depends on unstable compiler
features including [`generic_associated_types`], [`ptr_metadata`] and
[`unsize`].  Accordingly, a nightly toolchain is required.

## Example
```rust
use rattish::{coercible_trait, rtti_static, StaticDynCast};
use std::{any::Any, cell::RefCell, fmt, rc::Rc};

// casting from an object of trait Foo requires Foo to have super-trait Any..
trait Foo: Any {}
// ..and that Foo implements Coercible, for which there is a macro
coercible_trait!(Foo);

// casting to an object of trait Bar does not require anything special..
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

// ..except that Bar must be registered in the database with every one of its
//  concrete types that might get dynamically cast to a Bar trait object
rtti_static! {
    Bar: Qux,

    // example of another trait with multiple implementations
    fmt::LowerExp: u32 i32 f32,
}

// casting works through any type that implements Coercible
// implementations are provided for all standard pointer and wrapper types
// here, for example, are Rc and RefCell
let foo: Rc<RefCell<dyn Foo>> = Rc::new(RefCell::new(Qux(123)));

// explicit type annotation not required; only included here for information
let bar: Rc<RefCell<dyn Bar>> = foo.dyn_cast::<dyn Bar>().ok().unwrap();
let int = bar.borrow().bar();
assert_eq!(int, 246);

// enjoyed that?  have another, just for fun
let exp = (&int as &dyn Any).dyn_cast::<dyn fmt::LowerExp>().unwrap();
assert_eq!(format!("{:e}", exp), "2.46e2");
```

[`generic_associated_types`]: https://doc.rust-lang.org/nightly/unstable-book/language-features/generic-associated-types.html
[`ptr_metadata`]: https://doc.rust-lang.org/nightly/unstable-book/library-features/ptr-metadata.html
[`unsize`]: https://doc.rust-lang.org/nightly/unstable-book/library-features/unsize.html
