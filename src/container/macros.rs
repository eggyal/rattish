/// Implement [`Coercible`][super::Coercible] for the given trait, in order to
/// be able to cast *from* objects of that trait (requires that the trait have
/// [`Any`](core::any::Any) as a super-trait).
#[macro_export]
macro_rules! coercible_trait {
    ($trait:path) => {
        unsafe impl $crate::container::Coercible for dyn $trait {
            type Coerced<U: 'static + ?::core::marker::Sized> = U;
            type Innermost = Self;
        }

        unsafe impl $crate::container::InnermostTypeId for dyn $trait {
            fn innermost_type_id(&self) -> ::core::any::TypeId {
                ::core::any::Any::type_id(self)
            }
        }
    };
}

macro_rules! coercibles {
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
    (<$lt:lifetime, $x:ident, $u:ident>($self:ident) {
        $(#[$feature:literal])? $ty:ty => $coerced:ty { $($token:tt)+ },
        $($rest:tt)*
    }) => {
        $(
            #[cfg(any(feature = $feature, doc))]
            #[doc(cfg(feature = $feature))]
        )?
        unsafe impl<$lt, $x> $crate::container::InnermostTypeId for $ty
        where
            $x: ?::core::marker::Sized + $crate::container::InnermostTypeId,
        {
            fn innermost_type_id(&$self) -> ::core::any::TypeId {
                $($token)+
            }
        }

        coercibles! {
            <$lt, $x, $u>($self) {
                $(#[$feature])? $ty => $coerced {},
                $($rest)*
            }
        }
    };
    (<$lt:lifetime, $x:ident, $u:ident>($self:ident) {
        $(#[$feature:literal])? $ty:ty => $coerced:ty {},
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
        }

        coercibles! {
            <$lt, $x, $u>($self) {
                $($rest)*
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
            type Target = $x;

            unsafe fn coerce<U>($self, $metadata: $crate::container::Metadata<$crate::container::Coerced<Self::Target, U>>) -> Self::Coerced<U>
            where
                U: ?::core::marker::Sized,
                Self::Coerced<U>: ::core::marker::Sized,
            $body
        }
    )+)+};
}
