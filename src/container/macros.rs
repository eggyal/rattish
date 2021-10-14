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
            #[cfg_attr(feature = "tracing", $crate::tracing::instrument(skip_all))]
            fn innermost_type_id(
                &self,
            ) -> Result<::core::any::TypeId, $crate::container::TypeIdDeterminationError> {
                let type_id = ::core::any::Any::type_id(self);
                #[cfg(feature = "tracing")]
                $crate::tracing::info!("found type_id {:?}", type_id);
                Ok(type_id)
            }
        }
    };
}

macro_rules! coercibles {
    (
        <$t:ident, $u:ident>($self:ident, $metadata:ident) {
            $(#[$feature:literal])?
            $(@$lt:lifetime $tx:ty|)? $ty:ty => $coerced:ty $($coerce:block)? as _,
            $($rest:tt)*
        }
    ) => {
        coercibles! {
            <$t, $u>($self, $metadata) {
                $(#[$feature])?
                $(@$lt $tx|)? $ty => $coerced $($coerce)? as {
                    (**$self).innermost_type_id()
                },
                $($rest)*
            }
        }
    };
    (
        <$t:ident, $u:ident>($self:ident, $metadata:ident) {
            $(#[$feature:literal])?
            $(@$lt:lifetime $tx:ty|)? $ty:ty => $coerced:ty $($coerce:block)? as $type:block,
            $($rest:tt)*
        }
    ) => {
        $( #[cfg(feature = $feature)] )?
        unsafe impl<$t> $crate::container::InnermostTypeId for $ty
        where
            $t: ?::core::marker::Sized + $crate::container::InnermostTypeId,
        {
            #[cfg_attr(feature = "tracing", tracing::instrument(skip_all, fields(
                Self = ::core::any::type_name::<Self>(),
            )))]
            fn innermost_type_id(&$self) -> Result<::core::any::TypeId, $crate::container::TypeIdDeterminationError> $type
        }

        coercibles! {
            <$t, $u>($self, $metadata) {
                $(#[$feature])?
                $(@$lt $tx|)? $ty => $coerced $($coerce)?,
                $($rest)*
            }
        }
    };
    (
        <$t:ident, $u:ident>($self:ident, $metadata:ident) {
            $(#[$feature:literal])?
            $(@$lt:lifetime $tx:ty|)? $ty:ty => $coerced:ty $coerce:block,
            $($rest:tt)*
        }
    ) => {
        $( #[cfg(feature = $feature)] )?
        impl<$t> $crate::container::Pointer for $ty
        where
            $t: ?::core::marker::Sized + $crate::container::Coercible,
        {
            #[cfg_attr(feature = "tracing", tracing::instrument(skip_all, fields(
                Self = ::core::any::type_name::<Self>(),
                U = ::core::any::type_name::<U>(),
            )))]
            unsafe fn coerce<U>($self, $metadata: $crate::container::Metadata<$crate::container::Coerced<Self::Inner, U>>) -> Self::Coerced<U>
            where
                U: ?::core::marker::Sized,
                Self::Coerced<U>: ::core::marker::Sized,
            {
                #[allow(unused_unsafe)]
                unsafe {$coerce}
            }
        }

        coercibles! {
            <$t, $u>($self, $metadata) {
                $(#[$feature])?
                $(@$lt $tx|)? $ty => $coerced,
                $($rest)*
            }
        }
    };
    (
        <$t:ident, $u:ident>($self:ident, $metadata:ident) {
            $(#[$feature:literal])?
            @$lt:lifetime $tx:ty|$ty:ty => $coerced:ty,
            $($rest:tt)*
        }
    ) => {
        coercibles! {
            <$t, $u>($self, $metadata) {
                $(#[$feature])?
                @$lt $tx => $coerced,
                $($rest)*
            }
        }
    };
    (
        <$t:ident, $u:ident>($self:ident, $metadata:ident) {
            $(#[$feature:literal])?
            $(@$lt:lifetime)? $ty:ty => $coerced:ty,
            $($rest:tt)*
        }
    ) => {
        $( #[cfg(feature = $feature)] )?
        unsafe impl<$($lt,)? $t> $crate::container::Coercible for $ty
        where
            $t: ?::core::marker::Sized + $crate::container::Coercible,
        {
            type Coerced<$u: 'static + ?::core::marker::Sized> = $coerced;
            type Inner = $t;
            type Innermost = $t::Innermost;
        }

        coercibles! {
            <$t, $u>($self, $metadata) {
                $($rest)*
            }
        }
    };
    (<$t:ident, $u:ident>($self:ident, $metadata:ident) {}) => {};
}
