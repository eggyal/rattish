//! Database errors

use crate::container::TypeIdDeterminationError;
use core::{
    any::{type_name, TypeId},
    fmt,
    marker::PhantomData,
};

#[cfg(feature = "thiserror")]
use thiserror::Error;

/// Error that arose on accessing a database.
#[cfg_attr(feature = "thiserror", derive(Error))]
#[non_exhaustive]
pub enum DatabaseError<U>
where
    U: ?Sized,
{
    /// The database has not been initialized.
    #[cfg_attr(feature = "thiserror", error("database not initialized"))]
    NotInitialized,

    /// The `requested_type` is not registered in the database.
    #[cfg_attr(feature = "thiserror", error(
        "requested type <{}> not registered in database",
        type_name::<U>(),
    ))]
    RequestedTypeNotInDatabase {
        /// The type that was requested.
        requested_type: PhantomData<U>,
    },
}

impl<U> fmt::Debug for DatabaseError<U>
where
    U: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[allow(clippy::enum_glob_use)]
        use DatabaseError::*;

        #[allow(clippy::unneeded_field_pattern)]
        match *self {
            NotInitialized => write!(f, "NotInitialized"),

            RequestedTypeNotInDatabase { requested_type: _ } => f
                .debug_tuple("RequestedTypeNotInDatabase")
                .field(&type_name::<U>())
                .finish(),
        }
    }
}

/// Error that arose on accessing a database entry.
#[cfg_attr(feature = "thiserror", derive(Error))]
#[non_exhaustive]
pub enum DatabaseEntryError<U, P>
where
    U: 'static + ?Sized,
    P: ?Sized,
{
    /// The specified database error occurred.
    #[cfg_attr(feature = "thiserror", error(transparent))]
    DatabaseError {
        /// The database error.
        error: DatabaseError<U>,
    },

    /// The concrete type underlying the provided instance of `instance_type`
    /// could not be determined, for the specified `reason`.
    #[cfg_attr(feature = "thiserror", error(
        "unable to determine concrete type from provided instance of <{}>: {reason}",
        type_name::<P>(),
    ))]
    ConcreteTypeDeterminationFailure {
        /// The reason that the concrete type could not be determined.
        #[cfg_attr(feature = "thiserror", source)]
        reason: TypeIdDeterminationError,

        /// The pointer type.
        instance_type: PhantomData<P>,
    },

    /// The provided instance of `P` has the underlying concrete type with the
    /// specified `type_id`, but that type is not registered in the database for
    /// the `requested_type`.
    #[cfg_attr(feature = "thiserror", error(
        "provided instance of <{}> has concrete {type_id:?}, which is not registered in the database for target type <{}>",
        type_name::<P>(),
        type_name::<U>(),
    ))]
    ConcreteTypeNotRegisteredForTarget {
        /// The [`TypeId`] of the concrete type underlying the provided instance
        /// of `P`.
        type_id: TypeId,

        /// The type that was requested.
        requested_type: PhantomData<U>,

        /// The pointer type.
        instance_type: PhantomData<P>,
    },
}

impl<U, P> fmt::Debug for DatabaseEntryError<U, P>
where
    U: 'static + ?Sized,
    P: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[allow(clippy::enum_glob_use)]
        use DatabaseEntryError::*;

        #[allow(clippy::unneeded_field_pattern)]
        match *self {
            DatabaseError { ref error } => f.debug_tuple("DatabaseError").field(error).finish(),

            ConcreteTypeDeterminationFailure {
                ref reason,
                instance_type: _,
            } => f
                .debug_tuple("ConcreteTypeDeterminationFailure")
                .field(reason)
                .field(&type_name::<U>())
                .finish(),

            ConcreteTypeNotRegisteredForTarget {
                ref type_id,
                requested_type: _,
                instance_type: _,
            } => f
                .debug_tuple("ConcreteTypeNotRegisteredForTarget")
                .field(type_id)
                .field(&type_name::<U>())
                .field(&type_name::<P>())
                .finish(),
        }
    }
}

impl<U, P> From<DatabaseError<U>> for DatabaseEntryError<U, P>
where
    U: 'static + ?Sized,
    P: ?Sized,
{
    fn from(error: DatabaseError<U>) -> Self {
        Self::DatabaseError { error }
    }
}

impl<U, P> From<TypeIdDeterminationError> for DatabaseEntryError<U, P>
where
    U: 'static + ?Sized,
    P: ?Sized,
{
    fn from(reason: TypeIdDeterminationError) -> Self {
        Self::ConcreteTypeDeterminationFailure {
            reason,
            instance_type: PhantomData,
        }
    }
}

/// Error that arose on attempting to cast `pointer` to `U`.
#[cfg_attr(feature = "thiserror", derive(Error))]
#[cfg_attr(feature = "thiserror", error("{source}"))]
#[non_exhaustive]
pub struct CastError<U, P>
where
    U: 'static + ?Sized,
{
    /// The error that arose.
    pub source: DatabaseEntryError<U, P>,
    /// The (unmodified) pointer on which casting had been attempted, in order
    /// to return ownership back to the caller.
    pub pointer: P,
}

impl<U, P> fmt::Debug for CastError<U, P>
where
    U: 'static + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[allow(clippy::unneeded_field_pattern)]
        let Self {
            ref source,
            pointer: _,
        } = *self;
        f.debug_struct("Error")
            .field("source", source)
            .finish_non_exhaustive()
    }
}
