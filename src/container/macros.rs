/// Implement [`Coercible`][super::Coercible] for the given trait, in order to
/// be able to cast *from* objects of that trait (requires that the trait have
/// [`Any`](core::any::Any) as a super-trait).
#[macro_export]
macro_rules! coercible_trait {
    ($trait:path) => {
        unsafe impl $crate::container::Coercible for dyn $trait {
            type Coerced<U: 'static + ?Sized> = U;
            type Innermost = Self;
            fn innermost_type_id(&self) -> ::core::any::TypeId {
                ::core::any::Any::type_id(self)
            }
        }
    };
}

macro_rules! coercibles {
    (<$lt:lifetime, $x:ident, $u:ident>($self:ident) {
        $(#[$feature:literal])? $ty:ty => $coerced:ty $body:block,
        $($rest:tt)*
    }) => {
        $(
            #[cfg(any(feature = $feature, doc))]
            #[doc(cfg(feature = $feature))]
        )?
        unsafe impl<$lt, $x> $crate::container::Coercible for $ty
        where
            $x: ?::core::marker::Sized + $crate::container::Coercible,
        {
            type Coerced<$u: 'static + ?::core::marker::Sized> = $coerced;
            type Innermost = $x::Innermost;
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
            unsafe fn coerce<U>($self, $metadata: $crate::container::Metadata<$crate::container::Coerced<Self::Target, U>>) -> Self::Coerced<U>
            where
                U: ?Sized,
                Self::Coerced<U>: Sized,
            $body
        }
    )+)+};
}
