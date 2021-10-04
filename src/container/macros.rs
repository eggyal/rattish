macro_rules! coercibles {
    (<$lt:lifetime, $x:ident, $u:ident>($self:ident) {
        $(#[$feature:literal])? $ty:ty => $coerced:ty $body:block,
        $($rest:tt)*
    }) => {
        $(
            #[cfg(any(feature = $feature, doc))]
            #[doc(cfg(feature = $feature))]
        )?
        unsafe impl<$x> $crate::container::Coercible for $ty
        where
            $x: ?::core::marker::Sized + $crate::container::Coercible,
        {
            type Coerced<$lt, $u: $lt + ?::core::marker::Sized> = $coerced;
            type Innermost = $x::Innermost;

            #[inline(always)]
            fn innermost_type_id(&$self) -> ::core::any::TypeId $body
        }

        coercibles! {
            <$lt, $x, $u>($self) {
                $($rest)*
            }
        }
    };
    (<$lt:lifetime, $x:ident, $u:ident>($self:ident) {
        $(#[$feature:literal])? $ty:ty => $coerced:ty,
        $($rest:tt)*
    }) => {
        coercibles! {
            <$lt, $x, $u>($self) {
                $(#[$feature])? $ty => $coerced { (**$self).innermost_type_id() }, $($rest)*
            }
        }
    };
    (<$lt:lifetime, $x:ident, $u:ident>($self:ident) {}) => {};
}

macro_rules! pointers {
    ($(
        <$lt:lifetime, $x:ident>($self:ident, $metadata: ident) {$(
            $(#[$feature:literal])? $ty:ty $body:block
        )+}
    )+) => {$($(
        $(
            #[cfg(any(feature = $feature, doc))]
            #[doc(cfg(feature = $feature))]
        )?
        impl<$lt, $x> $crate::container::Pointer<$lt> for $ty
        where
            $x: ?::core::marker::Sized + $crate::container::Coercible,
        {
            unsafe fn coerce<U>($self, $metadata: $crate::container::Metadata<$crate::container::Coerced<'a, Self::Target, U>>) -> Self::Coerced<'a, U>
            where
                U: ?Sized,
                Self::Coerced<'a, U>: Sized,
            $body
        }
    )+)+};
}
