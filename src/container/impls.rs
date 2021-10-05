use core::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    ptr,
};

#[cfg(any(feature = "alloc", doc))]
use alloc::{boxed::Box, rc::Rc, sync::Arc};

coercible_trait!(Any);

coercibles! {
    <'a, T, U>(self) {
        *const T => *const T::Coerced<U> {},
        *mut T => *mut T::Coerced<U> {},
        &'a T => &'a T::Coerced<U>,
        &'a mut T => &'a mut T::Coerced<U>,
        RefCell<T> => RefCell<T::Coerced<U>> { self.borrow().innermost_type_id() },
        Ref<'a, T> => Ref<'a, T::Coerced<U>>,
        RefMut<'a, T> => RefMut<'a, T::Coerced<U>>,
        #["alloc"] Box<T> => Box<T::Coerced<U>>,
        #["alloc"] Rc<T> => Rc<T::Coerced<U>>,
        #["alloc"] Arc<T> => Arc<T::Coerced<U>>,
    }
}

pointers! {
    <'a, T>(self, metadata) {
        *const T {
            ptr::from_raw_parts(self.cast(), metadata)
        }
        *mut T {
            ptr::from_raw_parts_mut(self.cast(), metadata)
        }
        &'a T {
            &*(self as *const T).coerce(metadata)
        }
        &'a mut T {
            &mut *(self as *mut T).coerce(metadata)
        }
        Ref<'a, T> {
            Ref::map(self, |r| r.coerce(metadata))
        }
        RefMut<'a, T> {
            RefMut::map(self, |r| r.coerce(metadata))
        }
        #["alloc"] Box<T> {
            Box::from_raw(Box::into_raw(self).coerce(metadata))
        }
        #["alloc"] Rc<T> {
            Rc::from_raw(Rc::into_raw(self).coerce(metadata))
        }
        #["alloc"] Arc<T> {
            Arc::from_raw(Arc::into_raw(self).coerce(metadata))
        }
    }
}
