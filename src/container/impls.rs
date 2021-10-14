use core::{
    any::Any,
    cell::{Cell, Ref, RefCell, RefMut, UnsafeCell},
    ptr,
};

#[cfg(feature = "alloc")]
use super::TypeIdDeterminationError::UnableToUpgradeWeakReference;
#[cfg(feature = "alloc")]
use core::any::type_name;

#[cfg(all(feature = "alloc", not(any(feature = "std", doc))))]
use alloc::{boxed::Box, rc, sync};

#[cfg(any(feature = "std", doc))]
use std::{boxed::Box, rc, sync};

coercible_trait!(Any);

coercibles! {
    <T, U>(self, metadata) {
        *const T => *const T::Coerced<U> {
            ptr::from_raw_parts(self.cast(), metadata)
        },
        *mut T => *mut T::Coerced<U> {
            ptr::from_raw_parts_mut(self.cast(), metadata)
        },
        ptr::NonNull<T> => ptr::NonNull<T::Coerced<U>> {
            ptr::NonNull::from_raw_parts(self.cast(), metadata)
        },
        @'a &'a T|&T => &'a T::Coerced<U> {
            ptr::NonNull::from(self).coerce(metadata).as_ref()
        } as _,
        @'a &'a mut T|&mut T => &'a mut T::Coerced<U> {
            ptr::NonNull::from(self).coerce(metadata).as_mut()
        } as _,
        Cell<T> => Cell<T::Coerced<U>>,
        RefCell<T> => RefCell<T::Coerced<U>> as {
            self.borrow().innermost_type_id()
        },
        @'a Ref<'a, T>|Ref<'_, T> => Ref<'a, T::Coerced<U>> {
            Self::map(self, |r| r.coerce(metadata))
        } as _,
        @'a RefMut<'a, T>|RefMut<'_, T> => RefMut<'a, T::Coerced<U>> {
            Self::map(self, |r| r.coerce(metadata))
        } as _,
        UnsafeCell<T> => UnsafeCell<T::Coerced<U>>,
        #["alloc"] Box<T> => Box<T::Coerced<U>> {
            Box::from_raw(Self::into_raw(self).coerce(metadata))
        } as _,
        #["alloc"] rc::Rc<T> => rc::Rc<T::Coerced<U>> {
            rc::Rc::from_raw(Self::into_raw(self).coerce(metadata))
        } as _,
        #["alloc"] rc::Weak<T> => rc::Weak<T::Coerced<U>> {
            rc::Weak::from_raw(Self::into_raw(self).coerce(metadata))
        } as {
            self.upgrade()
                .ok_or(UnableToUpgradeWeakReference { type_name: type_name::<Self>() })?
                .innermost_type_id()
        },
        #["alloc"] sync::Arc<T> => sync::Arc<T::Coerced<U>> {
            sync::Arc::from_raw(Self::into_raw(self).coerce(metadata))
        } as _,
        #["alloc"] sync::Weak<T> => sync::Weak<T::Coerced<U>> {
            sync::Weak::from_raw(Self::into_raw(self).coerce(metadata))
        } as {
            self.upgrade()
                .ok_or(UnableToUpgradeWeakReference { type_name: type_name::<Self>() })?
                .innermost_type_id()
        },
    }
}
