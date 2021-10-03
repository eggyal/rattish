use super::{NonNullConst, NonNullMut};
use core::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    ops::DerefMut,
};

#[cfg(any(feature = "alloc", doc))]
use alloc::{boxed::Box, rc::Rc, sync::Arc};
#[cfg(feature = "alloc")]
use core::mem;

unsafe impl<'a> super::Coercible<'a> for dyn Any {
    type Coerced<U: 'a + ?Sized> = U;
    type Innermost = Self;

    #[inline(always)]
    fn innermost_type_id(&self) -> TypeId {
        self.type_id()
    }
}

coercibles! {
    <'a, T, U>(self) {
        &'a T => &'a T::Coerced<U>,
        &'a mut T => &'a T::Coerced<U>,
        RefCell<T> => RefCell<T::Coerced<U>> { self.borrow().innermost_type_id() },
        Ref<'_, T> => Ref<'a, T::Coerced<U>>,
        RefMut<'_, T> => RefMut<'a, T::Coerced<U>>,
        #["alloc"] Box<T> => Box<T::Coerced<U>>,
        #["alloc"] Rc<T> => Rc<T::Coerced<U>>,
        #["alloc"] Arc<T> => Arc<T::Coerced<U>>,
    }
}

recasts! {
    /// A [`PointerCoercer`][super::PointerCoercer] for [`Pointer`][super::Pointer]s that are created from immutable pointers.
    ConstCoercer<P, U> -> NonNullConst
    /// A [`PointerCoercer`][super::PointerCoercer] for [`Pointer`][super::Pointer]s that are created from mutable pointers.
    MutCoercer<P: DerefMut, U> -> NonNullMut
}

pointers! {
    ConstCoercer<'a, T>(self, ptr) {
        &'a T { ptr.as_ref() }
        Ref<'a, T> { Ref::map(self, |_| ptr.as_ref()) }
        #["alloc"] Rc<T> { mem::forget(self); Rc::from_raw(ptr.as_ref()) }
        #["alloc"] Arc<T> { mem::forget(self); Arc::from_raw(ptr.as_ref()) }
    }
    MutCoercer<'a, T>(self, mut ptr) {
        &'a mut T { ptr.as_mut() }
        RefMut<'a, T> { RefMut::map(self, |_| ptr.as_mut()) }
        #["alloc"] Box<T> { mem::forget(self); Box::from_raw(ptr.as_mut()) }
    }
}
