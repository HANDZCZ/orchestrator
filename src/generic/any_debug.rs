use std::{
    any::{self, Any},
    fmt::Debug,
};

/// Trait for adding debug to `Box<dyn Any>`. It also adds a method to get the type name of `Self` that's inside the `Box`.
pub trait AnyDebug: Any + Send + Sync + Debug + ToRefAny {
    /// Converts `Box<Self>` to `Box<dyn Any + Send + Sync>`.
    fn into_box_any(self: Box<Self>) -> Box<dyn Any + Send + Sync>;
    /// Returns the type name of `Self` thats inside the `Box` and `Box<dyn AnyDebug>`.
    fn with_type_name(self: Box<Self>) -> (&'static str, Box<dyn AnyDebug>);
}

impl<T> AnyDebug for T
where
    T: Any + Debug + Send + Sync,
{
    fn into_box_any(self: Box<Self>) -> Box<dyn Any + Send + Sync> {
        self
    }

    fn with_type_name(self: Box<Self>) -> (&'static str, Box<dyn AnyDebug>) {
        (any::type_name::<Self>(), self)
    }
}

pub trait ToRefAny {
    /// Converts `&Self` to `&(dyn Any + Send + Sync)`.
    ///
    /// ONLY USE THIS ON THE TYPE REFERENCE! (that means `&dyn ToRefAny`)
    ///
    /// Do not use it on `&Box<dyn ToRefAny>` and similar.
    /// You won't be able to downcast it.
    fn to_ref_any(&self) -> &(dyn Any + Send + Sync);
    /// Converts `&mut Self` to `&mut (dyn Any + Send + Sync)`.
    ///
    /// ONLY USE THIS ON THE TYPE REFERENCE! (that means `&mut dyn ToRefAny`)
    ///
    /// Do not use it on `&mut Box<dyn ToRefAny>` and similar.
    /// You won't be able to downcast it.
    fn to_mut_ref_any(&mut self) -> &mut (dyn Any + Send + Sync);
}

impl<T> ToRefAny for T
where
    T: Any + Send + Sync,
{
    fn to_ref_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }

    fn to_mut_ref_any(&mut self) -> &mut (dyn Any + Send + Sync) {
        self
    }
}
