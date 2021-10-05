use core::{
    any::Any,
    cell::{Cell, Ref, RefCell, RefMut, UnsafeCell},
    ptr,
};

#[cfg(any(feature = "alloc", doc))]
use alloc::{boxed::Box, rc, sync};

coercible_trait!(Any);

coercibles! {
    <'a, T, U>(self, metadata) {
        *const T => *const T::Coerced<U> {
            ptr::from_raw_parts(self.cast(), metadata)
        },
        *mut T => *mut T::Coerced<U> {
            ptr::from_raw_parts_mut(self.cast(), metadata)
        },
        ptr::NonNull<T> => ptr::NonNull<T::Coerced<U>> {
            ptr::NonNull::from_raw_parts(self.cast(), metadata)
        },
        &'a T => &'a T::Coerced<U> {
            ptr::NonNull::from(self).coerce(metadata).as_ref()
        } as _,
        &'a mut T => &'a mut T::Coerced<U> {
            ptr::NonNull::from(self).coerce(metadata).as_mut()
        } as _,
        Cell<T> => Cell<T::Coerced<U>>,
        RefCell<T> => RefCell<T::Coerced<U>> as {
            self.borrow().innermost_type_id()
        },
        Ref<'a, T> => Ref<'a, T::Coerced<U>> {
            Ref::map(self, |r| r.coerce(metadata))
        } as _,
        RefMut<'a, T> => RefMut<'a, T::Coerced<U>> {
            RefMut::map(self, |r| r.coerce(metadata))
        } as _,
        UnsafeCell<T> => UnsafeCell<T::Coerced<U>>,
        #["alloc"] Box<T> => Box<T::Coerced<U>> {
            Box::from_raw(Box::into_raw(self).coerce(metadata))
        } as _,
        #["alloc"] rc::Rc<T> => rc::Rc<T::Coerced<U>> {
            rc::Rc::from_raw(rc::Rc::into_raw(self).coerce(metadata))
        } as _,
        #["alloc"] rc::Weak<T> => rc::Weak<T::Coerced<U>> {
            rc::Weak::from_raw(rc::Weak::into_raw(self).coerce(metadata))
        } as {
            self.upgrade()?.innermost_type_id()
        },
        #["alloc"] sync::Arc<T> => sync::Arc<T::Coerced<U>> {
            sync::Arc::from_raw(sync::Arc::into_raw(self).coerce(metadata))
        } as _,
        #["alloc"] sync::Weak<T> => sync::Weak<T::Coerced<U>> {
            sync::Weak::from_raw(sync::Weak::into_raw(self).coerce(metadata))
        } as {
            self.upgrade()?.innermost_type_id()
        },
    }
}
