use super::{InnermostTypeId, Metadata, Pointer};
use core::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    cmp,
    marker::Unsize,
    ptr,
};

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, rc, sync};

#[cfg(feature = "alloc")]
use super::TypeIdDeterminationError::UnableToUpgradeWeakReference;

type T = i32;
type U = dyn cmp::PartialEq<T>;
const ADDRESS: *mut () = 0xdeadbeef_usize as _;
const METADATA: Metadata<U> = ptr::metadata::<U>(ptr::null::<T>());

trait Foo {
    fn double(&mut self);
    fn get(&self) -> i32;
}
impl Foo for i32 {
    fn double(&mut self) {
        *self *= 2;
    }
    fn get(&self) -> i32 {
        *self
    }
}

fn metadata<T: Unsize<U>, U: ?Sized>() -> Metadata<U> {
    ptr::metadata::<U>(ptr::null::<T>())
}

#[test]
fn raw_const_ptr_coerces() {
    const ADDRESS: *const () = self::ADDRESS as _;
    unsafe {
        let ptr: *const dyn Any = ADDRESS as *const T;
        let coerced = ptr.coerce::<U>(METADATA);

        assert_eq!(coerced.to_raw_parts(), (ADDRESS, METADATA));
    }
}

#[test]
fn raw_mut_ptr_coerces() {
    unsafe {
        let ptr: *mut dyn Any = ADDRESS as *mut T;
        let coerced = ptr.coerce::<U>(METADATA);

        assert_eq!(coerced.to_raw_parts(), (ADDRESS, METADATA));
    }
}

#[test]
fn non_null_coerces() {
    unsafe {
        let ptr: ptr::NonNull<dyn Any> = ptr::NonNull::new_unchecked(ADDRESS as *mut T);
        let coerced = ptr.coerce::<U>(METADATA);

        assert_eq!(
            coerced.to_raw_parts(),
            (ptr::NonNull::new_unchecked(ADDRESS), METADATA)
        );
    }
}

#[test]
fn ref_coerces() {
    unsafe {
        let ptr: &dyn Any = &12345;
        let coerced = ptr.coerce::<U>(METADATA);

        assert!(coerced.eq(&12345));
    }
}

#[test]
fn mut_ref_coerces() {
    unsafe {
        let ptr: &mut dyn Any = &mut 12345;
        let coerced = ptr.coerce::<dyn Foo>(metadata::<T, dyn Foo>());

        coerced.double();
        assert_eq!(coerced.get(), 12345 * 2);
    }
}

#[test]
fn cell_ref_coerces() {
    unsafe {
        let cell = RefCell::new(12345);
        let borrow: Ref<dyn Any> = cell.borrow();
        let coerced = borrow.coerce::<U>(METADATA);

        assert!(coerced.eq(&12345));
    }
}

#[test]
fn cell_refmut_coerces() {
    unsafe {
        let cell = RefCell::new(12345);
        let borrow: RefMut<dyn Any> = cell.borrow_mut();
        let mut coerced = borrow.coerce::<dyn Foo>(metadata::<T, dyn Foo>());
        coerced.double();

        assert_eq!(coerced.get(), 12345 * 2);
    }
}

#[cfg(feature = "alloc")]
#[test]
fn box_coerces() {
    unsafe {
        let boxed: Box<dyn Any> = Box::new(12345);
        let coerced = boxed.coerce::<U>(METADATA);

        assert!(coerced.eq(&12345));
    }
}

#[cfg(feature = "alloc")]
#[test]
fn strong_rc_coerces() {
    unsafe {
        let rc: rc::Rc<dyn Any> = rc::Rc::new(12345);
        let coerced = rc.coerce::<U>(METADATA);

        assert!(coerced.eq(&12345));
    }
}

#[cfg(feature = "alloc")]
#[test]
fn weak_rc_coerces() {
    unsafe {
        let rc = rc::Rc::new(12345);
        let weak: rc::Weak<dyn Any> = rc::Rc::downgrade(&rc) as _;
        let coerced = weak.coerce::<U>(METADATA);

        assert!(coerced.upgrade().unwrap().eq(&12345));
    }
}

#[cfg(feature = "alloc")]
#[test]
fn weak_rc_coerces_even_if_dangling() {
    unsafe {
        let weak: rc::Weak<dyn Any> = rc::Rc::downgrade(&rc::Rc::new(12345)) as _;
        let coerced = weak.coerce::<U>(METADATA);

        assert!(coerced.upgrade().is_none());
    }
}

#[cfg(feature = "alloc")]
#[test]
fn strong_arc_coerces() {
    unsafe {
        let arc: sync::Arc<dyn Any> = sync::Arc::new(12345);
        let coerced = arc.coerce::<U>(METADATA);

        assert!(coerced.eq(&12345));
    }
}

#[cfg(feature = "alloc")]
#[test]
fn weak_arc_coerces() {
    unsafe {
        let arc = sync::Arc::new(12345);
        let weak: sync::Weak<dyn Any> = sync::Arc::downgrade(&arc) as _;
        let coerced = weak.coerce::<U>(METADATA);

        assert!(coerced.upgrade().unwrap().eq(&12345));
    }
}

#[cfg(feature = "alloc")]
#[test]
fn weak_arc_coerces_even_if_dangling() {
    unsafe {
        let weak: sync::Weak<dyn Any> = sync::Arc::downgrade(&sync::Arc::new(12345)) as _;
        let coerced = weak.coerce::<U>(METADATA);

        assert!(coerced.upgrade().is_none());
    }
}

#[test]
fn compound_types_transitively_coerce() {
    unsafe {
        let cell = RefCell::new(12345);
        let compound: &RefCell<dyn Any> = &cell;
        let coerced = compound.coerce::<U>(METADATA);

        assert!(coerced.borrow().eq(&12345));
    }
}

#[test]
fn innermost_type_id_of_ref() {
    let ptr: &dyn Any = &12345;
    let type_id = ptr.innermost_type_id().unwrap();

    assert_eq!(type_id, TypeId::of::<i32>());
}

#[test]
fn innermost_type_id_of_mut_ref() {
    let ptr: &mut dyn Any = &mut 12345;
    let type_id = ptr.innermost_type_id().unwrap();

    assert_eq!(type_id, TypeId::of::<i32>());
}

#[test]
fn innermost_type_id_of_cell_ref() {
    let cell = RefCell::new(12345);
    let borrow: Ref<dyn Any> = cell.borrow();
    let type_id = borrow.innermost_type_id().unwrap();

    assert_eq!(type_id, TypeId::of::<i32>());
}

#[test]
fn innermost_type_id_of_cell_refmut() {
    let cell = RefCell::new(12345);
    let borrow: RefMut<dyn Any> = cell.borrow_mut();
    let type_id = borrow.innermost_type_id().unwrap();

    assert_eq!(type_id, TypeId::of::<i32>());
}

#[cfg(feature = "alloc")]
#[test]
fn innermost_type_id_of_box() {
    let boxed: Box<dyn Any> = Box::new(12345);
    let type_id = boxed.innermost_type_id().unwrap();

    assert_eq!(type_id, TypeId::of::<i32>());
}

#[cfg(feature = "alloc")]
#[test]
fn innermost_type_id_of_strong_rc() {
    let rc: rc::Rc<dyn Any> = rc::Rc::new(12345);
    let type_id = rc.innermost_type_id().unwrap();

    assert_eq!(type_id, TypeId::of::<i32>());
}

#[cfg(feature = "alloc")]
#[test]
fn innermost_type_id_of_weak_rc() {
    let rc = rc::Rc::new(12345);
    let weak: rc::Weak<dyn Any> = rc::Rc::downgrade(&rc) as _;
    let type_id = weak.innermost_type_id().unwrap();

    assert_eq!(type_id, TypeId::of::<i32>());
}

#[cfg(feature = "alloc")]
#[test]
fn innermost_type_id_of_weak_rc_fails_if_dangling() {
    let weak: rc::Weak<dyn Any> = rc::Rc::downgrade(&rc::Rc::new(12345)) as _;
    let type_id = weak.innermost_type_id();

    assert_eq!(
        type_id,
        Err(UnableToUpgradeWeakReference {
            type_name: "alloc::rc::Weak<dyn core::any::Any>",
        })
    );
}

#[cfg(feature = "alloc")]
#[test]
fn innermost_type_id_of_strong_arc() {
    let arc: sync::Arc<dyn Any> = sync::Arc::new(12345);
    let type_id = arc.innermost_type_id().unwrap();

    assert_eq!(type_id, TypeId::of::<i32>());
}

#[cfg(feature = "alloc")]
#[test]
fn innermost_type_id_of_weak_arc() {
    let arc = sync::Arc::new(12345);
    let weak: sync::Weak<dyn Any> = sync::Arc::downgrade(&arc) as _;
    let type_id = weak.innermost_type_id().unwrap();

    assert_eq!(type_id, TypeId::of::<i32>());
}

#[cfg(feature = "alloc")]
#[test]
fn innermost_type_id_of_weak_arc_fails_if_dangling() {
    let weak: sync::Weak<dyn Any> = sync::Arc::downgrade(&sync::Arc::new(12345)) as _;
    let type_id = weak.innermost_type_id();

    assert_eq!(
        type_id,
        Err(UnableToUpgradeWeakReference {
            type_name: "alloc::sync::Weak<dyn core::any::Any>",
        })
    );
}

#[test]
fn innermost_type_id_of_compound_types_are_transitive() {
    let cell = RefCell::new(12345);
    let compound: &RefCell<dyn Any> = &cell;
    let type_id = compound.innermost_type_id().unwrap();

    assert_eq!(type_id, TypeId::of::<i32>());
}
