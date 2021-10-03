macro_rules! castables {
    (<$lt:lifetime, $x:ident, $u:ident>($self:ident) {
        $(#[$feature:literal])? $ty:ty => $casted:ty $body:block,
        $($rest:tt)*
    }) => {
        $(
            #[cfg(any(feature = $feature, doc))]
            #[doc(cfg(feature = $feature))]
        )?
        unsafe impl<$lt, $x> $crate::container::Coercible<'a> for $ty
        where
            $x: ?::core::marker::Sized + $crate::container::Coercible<'a>,
        {
            type Coerced<$u: $lt + ?::core::marker::Sized> = $casted;
            type Innermost = $x::Innermost;

            #[inline(always)]
            fn innermost_type_id(&$self) -> ::core::any::TypeId $body
        }

        castables! {
            <$lt, $x, $u>($self) {
                $($rest)*
            }
        }
    };
    (<$lt:lifetime, $x:ident, $u:ident>($self:ident) {
        $(#[$feature:literal])? $ty:ty => $casted:ty,
        $($rest:tt)*
    }) => {
        castables! {
            <$lt, $x, $u>($self) {
                $(#[$feature])? $ty => $casted { (**$self).innermost_type_id() }, $($rest)*
            }
        }
    };
    (<$lt:lifetime, $x:ident, $u:ident>($self:ident) {}) => {};
}

macro_rules! recasts {
    ($($(#[$attr:meta])* $coercer:ident<$p:ident$(: $trait:path)?, $u:ident> -> $ty:ident)+) => {$(
        $(#[$attr])*
        pub struct $coercer;
        impl<'a, $p> $crate::container::PointerCoercer<'a, $p> for $coercer
        where
            $p: $crate::container::Pointer<'a, Coercer = Self> $(+ $trait)?,
            $p::Target: $crate::container::Coercible<'a>,
        {
            type Raw<$u: ?::core::marker::Sized> = $ty<$u>;

            #[inline(always)]
            unsafe fn coerce<$u>(
                pointer: $p,
                metadata: $crate::container::Metadata<$crate::container::Coerced<'a, $p::Target, $u>>,
            ) -> $p::Coerced<U>
            where
                $u: ?::core::marker::Sized,
                $p::Coerced<$u>: ::core::marker::Sized,
            {
                let data_address = $ty::from(&*pointer).cast();
                let ptr = $ty::from_raw_parts(data_address, metadata);
                pointer.repoint(ptr)
            }
        }
    )+};
}

macro_rules! pointers {
    ($(
        $coercer:ident<$lt:lifetime, $x:ident>($self:ident, $ptr:pat) {$(
            $(#[$feature:literal])? $ty:ty $body:block
        )+}
    )+) => {$($(
        $(
            #[cfg(any(feature = $feature, doc))]
            #[doc(cfg(feature = $feature))]
        )?
        impl<$lt, $x> $crate::container::Pointer<$lt> for $ty
        where
            $x: ?::core::marker::Sized + $crate::container::Coercible<$lt>,
        {
            type Coercer = $coercer;

            #[inline(always)]
            unsafe fn repoint<U>($self, $ptr: $crate::container::PointerRaw<'a, Self, U>) -> Self::Coerced<U>
            where
                U: ?::core::marker::Sized,
                Self::Coerced<U>: ::core::marker::Sized,
            $body
        }
    )+)+};
}
