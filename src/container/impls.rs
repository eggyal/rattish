use core::{
    any::Any,
    cell::{Cell, Ref, RefCell, RefMut, UnsafeCell},
    ptr,
};

#[cfg(feature = "alloc")]
use super::TypeIdDeterminationError::UnableToUpgradeWeakReference;
#[cfg(feature = "alloc")]
use core::any::type_name;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{boxed::Box, rc, sync};

#[cfg(feature = "std")]
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
        Cell<T> => Cell<T::Coerced<U>>/* as {
            unsafe { &*self.as_ptr() }.innermost_type_id()
        }*/,
        RefCell<T> => RefCell<T::Coerced<U>> as {
            self.borrow().innermost_type_id()
        },
        @'a Ref<'a, T>|Ref<'_, T> => Ref<'a, T::Coerced<U>> = mapped as _,
        @'a RefMut<'a, T>|RefMut<'_, T> => RefMut<'a, T::Coerced<U>> = mapped as _,
        UnsafeCell<T> => UnsafeCell<T::Coerced<U>>/* as {
            unsafe { &*self.get() }.innermost_type_id()
        }*/,
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

        // We cannot permit traversal of synchronous locks (Mutex, RwLock, etc) because
        // this can lead to unsoundness: e.g. if innermost type of &Mutex is identified
        // but then changed by another thread (this is not the case for RefCell because
        // it is necessarily single-threaded). We can however permit traversal of their
        // locks; however, std library locks do not provide for mapping - parking_lot's
        // do, implementations for which we also provide.
        #["std"] sync::Mutex<T> => sync::Mutex<T::Coerced<U>>,
        #["std"] @'a sync::MutexGuard<'a, T>|sync::MutexGuard<'_, T> => sync::MutexGuard<'a, T::Coerced<U>> as _,
        #["std"] sync::RwLock<T> => sync::RwLock<T::Coerced<U>>,
        #["std"] @'a sync::RwLockReadGuard<'a, T>|sync::RwLockReadGuard<'_, T> => sync::RwLockReadGuard<'a, T::Coerced<U>> as _,
        #["std"] @'a sync::RwLockWriteGuard<'a, T>|sync::RwLockWriteGuard<'_, T> => sync::RwLockWriteGuard<'a, T::Coerced<U>> as _,
        #["parking_lot"] parking_lot::Mutex<T> => parking_lot::Mutex<T::Coerced<U>>,
        #["parking_lot"] parking_lot::FairMutex<T> => parking_lot::FairMutex<T::Coerced<U>>,
        #["parking_lot"] parking_lot::ReentrantMutex<T> => parking_lot::ReentrantMutex<T::Coerced<U>>,
        #["parking_lot"] parking_lot::RwLock<T> => parking_lot::RwLock<T::Coerced<U>>,
        #["parking_lot"] @'a parking_lot::MutexGuard<'a, T>|parking_lot::MutexGuard<'_, T> => parking_lot::MappedMutexGuard<'a, T::Coerced<U>> = mapped as _,
        #["parking_lot"] @'a parking_lot::FairMutexGuard<'a, T>|parking_lot::FairMutexGuard<'_, T> => parking_lot::MappedFairMutexGuard<'a, T::Coerced<U>> = mapped as _,
        #["parking_lot"] @'a parking_lot::ReentrantMutexGuard<'a, T>|parking_lot::ReentrantMutexGuard<'_, T> => parking_lot::MappedReentrantMutexGuard<'a, T::Coerced<U>> = mapped as _,
        #["parking_lot"] @'a parking_lot::RwLockReadGuard<'a, T>|parking_lot::RwLockReadGuard<'_, T> => parking_lot::MappedRwLockReadGuard<'a, T::Coerced<U>> = mapped as _,
        #["parking_lot"] @'a parking_lot::RwLockWriteGuard<'a, T>|parking_lot::RwLockWriteGuard<'_, T> => parking_lot::MappedRwLockWriteGuard<'a, T::Coerced<U>> = mapped as _,
        #["parking_lot"] @'a parking_lot::RwLockUpgradableReadGuard<'a, T>|parking_lot::RwLockUpgradableReadGuard<'_, T> => parking_lot::RwLockUpgradableReadGuard<'a, T::Coerced<U>> as _,
    }
}
