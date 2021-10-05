use core::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    ptr,
};

#[cfg(any(feature = "alloc", doc))]
use alloc::{boxed::Box, rc::Rc, sync::Arc};

coercible_trait!(Any);

coercibles! {
    <'a, T, U>(self, metadata) {
        *const T => *const T::Coerced<U> {
            ptr::from_raw_parts(self.cast(), metadata)
        },
        *mut T => *mut T::Coerced<U> {
            ptr::from_raw_parts_mut(self.cast(), metadata)
        },
        &'a T => &'a T::Coerced<U> {
            &*(self as *const T).coerce(metadata)
        } as _,
        &'a mut T => &'a mut T::Coerced<U> {
            &mut *(self as *mut T).coerce(metadata)
        } as _,
        RefCell<T> => RefCell<T::Coerced<U>> as {
            self.borrow().innermost_type_id()
        },
        Ref<'a, T> => Ref<'a, T::Coerced<U>> {
            Ref::map(self, |r| r.coerce(metadata))
        } as _,
        RefMut<'a, T> => RefMut<'a, T::Coerced<U>> {
            RefMut::map(self, |r| r.coerce(metadata))
        } as _,
        #["alloc"] Box<T> => Box<T::Coerced<U>> {
            Box::from_raw(Box::into_raw(self).coerce(metadata))
        } as _,
        #["alloc"] Rc<T> => Rc<T::Coerced<U>> {
            Rc::from_raw(Rc::into_raw(self).coerce(metadata))
        } as _,
        #["alloc"] Arc<T> => Arc<T::Coerced<U>> {
            Arc::from_raw(Arc::into_raw(self).coerce(metadata))
        } as _,
    }
}
