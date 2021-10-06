use super::{
    error::{
        CastError,
        DatabaseEntryError::{
            ConcreteTypeDeterminationFailure, ConcreteTypeNotRegisteredForTarget,
        },
        DatabaseError::RequestedTypeNotInDatabase,
    },
    hash_map::HashMapTypeDatabase,
    TypeDatabaseEntryExt, TypeDatabaseExt,
};
use crate::rtti;
use std::{any::Any, cmp, lazy::SyncLazy, rc};

static DB: SyncLazy<HashMapTypeDatabase> = SyncLazy::new(|| {
    rtti! {
        cmp::PartialEq<i32>: i32,
        cmp::PartialEq<f32>: f32,
    }
});

#[test]
fn db_has_registered_targets() {
    let target = DB.get_db_entry::<dyn cmp::PartialEq<i32>>();
    assert!(target.is_ok());
}

#[test]
fn db_does_not_have_unregistered_targets() {
    let target = DB.get_db_entry::<dyn cmp::PartialEq<u32>>();
    assert!(matches!(target, Err(RequestedTypeNotInDatabase { .. })));
}

#[test]
fn targets_implement_registered_types() {
    let target = DB.get_db_entry::<dyn cmp::PartialEq<i32>>().unwrap();
    assert!(target.implements(&0i32 as &dyn Any).unwrap());
}

#[test]
fn targets_do_not_implement_unregistered_types() {
    let target = DB.get_db_entry::<dyn cmp::PartialEq<i32>>().unwrap();
    assert!(!target.implements(&0f32 as &dyn Any).unwrap());
}

#[test]
fn targets_cannot_determine_implementation_of_dangling_weak_rc() {
    let target = DB.get_db_entry::<dyn cmp::PartialEq<i32>>().unwrap();
    let weak: rc::Weak<dyn Any> = rc::Rc::downgrade(&rc::Rc::new(12345)) as _;

    assert!(matches!(
        target.implements(&weak),
        Err(ConcreteTypeDeterminationFailure { .. }),
    ));
}

#[test]
fn registered_type_is_casted() {
    let target = DB.get_db_entry::<dyn cmp::PartialEq<i32>>().unwrap();
    let casted = target.cast(&12345i32 as &dyn Any).unwrap();

    assert!(casted.eq(&12345));
}

#[test]
fn unregistered_type_is_not_casted() {
    let target = DB.get_db_entry::<dyn cmp::PartialEq<i32>>().unwrap();
    let casted = target.cast(&12345f32 as &dyn Any);

    assert!(matches!(
        casted,
        Err(CastError {
            source: ConcreteTypeNotRegisteredForTarget { .. },
            ..
        })
    ));
}

#[test]
fn cannot_cast_dangling_weak_rc() {
    let target = DB.get_db_entry::<dyn cmp::PartialEq<i32>>().unwrap();
    let weak: rc::Weak<dyn Any> = rc::Rc::downgrade(&rc::Rc::new(12345)) as _;
    let casted = target.cast(weak);

    assert!(matches!(
        casted,
        Err(CastError {
            source: ConcreteTypeDeterminationFailure { .. },
            ..
        })
    ));
}
