/// Implement [`Coercible`][super::Coercible] for the given trait, in order to
/// be able to cast *from* objects of that trait (requires that the trait have
/// [`Any`](core::any::Any) as a super-trait).
#[macro_export]
macro_rules! coercible_trait {
    ($trait:path) => {
        unsafe impl $crate::container::Coercible for dyn $trait {
            type Coerced<U: 'static + ?::core::marker::Sized> = U;
            type Inner = Self;
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
    (
        <$lt:lifetime, $t:ident, $u:ident>($self:ident, $metadata:ident) {
            $(#[$feature:literal])? $ty:ty => $coerced:ty $($coerce:block)? as _,
            $($rest:tt)*
        }
    ) => {
        coercibles! {
            <$lt, $t, $u>($self, $metadata) {
                $(#[$feature])? $ty => $coerced $($coerce)? as { (**$self).innermost_type_id() },
                $($rest)*
            }
        }
    };
    (
        <$lt:lifetime, $t:ident, $u:ident>($self:ident, $metadata:ident) {
            $(#[$feature:literal])? $ty:ty => $coerced:ty $($coerce:block)? as $type:block,
            $($rest:tt)*
        }
    ) => {
        $(
            #[cfg(any(feature = $feature, doc))]
            #[doc(cfg(feature = $feature))]
        )?
        unsafe impl<$lt, $t> $crate::container::InnermostTypeId for $ty
        where
            $t: ?::core::marker::Sized + $crate::container::InnermostTypeId,
        {
            fn innermost_type_id(&$self) -> ::core::any::TypeId $type
        }

        coercibles! {
            <$lt, $t, $u>($self, $metadata) {
                $(#[$feature])? $ty => $coerced $($coerce)?,
                $($rest)*
            }
        }
    };
    (
        <$lt:lifetime, $t:ident, $u:ident>($self:ident, $metadata:ident) {
            $(#[$feature:literal])? $ty:ty => $coerced:ty $coerce:block,
            $($rest:tt)*
        }
    ) => {
        $(
            #[cfg(any(feature = $feature, doc))]
            #[doc(cfg(feature = $feature))]
        )?
        impl<$lt, $t> $crate::container::Pointer for $ty
        where
            $t: ?::core::marker::Sized + $crate::container::Coercible,
        {
            unsafe fn coerce<U>($self, $metadata: $crate::container::Metadata<$crate::container::Coerced<Self::Inner, U>>) -> Self::Coerced<U>
            where
                U: ?::core::marker::Sized,
                Self::Coerced<U>: ::core::marker::Sized,
            $coerce
        }

        coercibles! {
            <$lt, $t, $u>($self, $metadata) {
                $(#[$feature])? $ty => $coerced,
                $($rest)*
            }
        }
    };
    (
        <$lt:lifetime, $t:ident, $u:ident>($self:ident, $metadata:ident) {
            $(#[$feature:literal])? $ty:ty => $coerced:ty,
            $($rest:tt)*
        }
    ) => {
        $(
            #[cfg(any(feature = $feature, doc))]
            #[doc(cfg(feature = $feature))]
        )?
        unsafe impl<$lt, $t> $crate::container::Coercible for $ty
        where
            $t: ?::core::marker::Sized + $crate::container::Coercible,
        {
            type Coerced<$u: 'static + ?::core::marker::Sized> = $coerced;
            type Inner = $t;
            type Innermost = $t::Innermost;
        }

        coercibles! {
            <$lt, $t, $u>($self, $metadata) {
                $($rest)*
            }
        }
    };
    (<$lt:lifetime, $t:ident, $u:ident>($self:ident, $metadata:ident) {}) => {};
}
