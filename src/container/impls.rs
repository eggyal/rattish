use super::{Coerce, Metadata};
use core::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    ptr,
};

#[cfg(any(feature = "alloc", doc))]
use alloc::{boxed::Box, rc::Rc, sync::Arc};

impl<T> Coerce for T
where
    T: ?Sized,
{
    #[inline(always)]
    unsafe fn coerce_ref<U>(&self, metadata: Metadata<U>) -> &U
    where
        U: ?Sized,
    {
        let data_address = (self as *const Self).cast();
        let ptr = ptr::from_raw_parts(data_address, metadata);
        &*ptr
    }

    #[inline(always)]
    unsafe fn coerce_mut<U>(&mut self, metadata: Metadata<U>) -> &mut U
    where
        U: ?Sized,
    {
        let data_address = (self as *mut Self).cast();
        let ptr = ptr::from_raw_parts_mut(data_address, metadata);
        &mut *ptr
    }
}

coercible_trait!(Any);

coercibles! {
    <'a, T, U>(self) {
        &T => &'a T::Coerced<'a, U>,
        &mut T => &'a T::Coerced<'a, U>,
        RefCell<T> => RefCell<T::Coerced<'a, U>> { self.borrow().innermost_type_id() },
        Ref<'_, T> => Ref<'a, T::Coerced<'a, U>>,
        RefMut<'_, T> => RefMut<'a, T::Coerced<'a, U>>,
        #["alloc"] Box<T> => Box<T::Coerced<'a, U>>,
        #["alloc"] Rc<T> => Rc<T::Coerced<'a, U>>,
        #["alloc"] Arc<T> => Arc<T::Coerced<'a, U>>,
    }
}

pointers! {
    <'a, T>(self, metadata) {
        &'a T { self.coerce_ref(metadata) }
        &'a mut T { self.coerce_mut(metadata) }
        Ref<'a, T> { Ref::map(self, |r| r.coerce_ref(metadata)) }
        RefMut<'a, T> { RefMut::map(self, |r| r.coerce_mut(metadata)) }
        #["alloc"] Box<T> { Box::from_raw(Box::leak(self).coerce_mut(metadata)) }
        #["alloc"] Rc<T> { Rc::from_raw((&*Rc::into_raw(self)).coerce_ref(metadata)) }
        #["alloc"] Arc<T> { Arc::from_raw((&*Arc::into_raw(self)).coerce_ref(metadata)) }
    }
}
