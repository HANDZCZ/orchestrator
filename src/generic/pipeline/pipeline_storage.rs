use std::{any::TypeId, collections::HashMap, fmt::Debug};

use crate::generic::AnyDebug;

#[derive(Debug, Default)]
pub struct PipelineStorage {
    inner: HashMap<TypeId, Box<dyn AnyDebug>>,
}

impl PipelineStorage {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    // should never panic
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn get<T>(&self) -> Option<&T>
    where
        T: AnyDebug + 'static,
    {
        self.inner.get(&TypeId::of::<T>()).map(|val| {
            let any_debug_ref = &**val;
            any_debug_ref.to_ref_any().downcast_ref::<T>().unwrap()
        })
    }

    // should never panic
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn get_mut<T>(&mut self) -> Option<&mut T>
    where
        T: AnyDebug + 'static,
    {
        self.inner.get_mut(&TypeId::of::<T>()).map(|val| {
            let any_debug_ref = &mut **val;
            any_debug_ref.to_mut_ref_any().downcast_mut::<T>().unwrap()
        })
    }

    // should never panic
    #[allow(clippy::missing_panics_doc)]
    pub fn insert<T>(&mut self, val: T) -> Option<T>
    where
        T: AnyDebug + 'static,
    {
        self.inner
            .insert(TypeId::of::<T>(), Box::new(val))
            .map(|val| *val.into_box_any().downcast::<T>().unwrap())
    }

    // should never panic
    #[allow(clippy::missing_panics_doc)]
    pub fn remove<T>(&mut self) -> Option<T>
    where
        T: AnyDebug + 'static,
    {
        self.inner
            .remove(&TypeId::of::<T>())
            .map(|val| *val.into_box_any().downcast::<T>().unwrap())
    }
}

#[test]
fn works() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct MyVal(String);

    let mut s = PipelineStorage::new();
    s.insert(MyVal("test".into()));
    //println!("{s:#?}");
    let v = s.get::<MyVal>();
    assert!(v.is_some());
    assert_eq!(v.unwrap().0, "test".to_string());

    let v = s.get_mut::<MyVal>();
    assert!(v.is_some());
    assert_eq!(v.as_ref().unwrap().0, "test".to_string());
    *v.unwrap() = MyVal("hmm".into());

    let v = s.insert(MyVal("jop".into()));
    assert!(v.is_some());
    assert_eq!(v.unwrap().0, "hmm".to_string());

    let v = s.remove::<MyVal>();
    assert!(v.is_some());
    assert_eq!(v.unwrap().0, "jop".to_string());
}
