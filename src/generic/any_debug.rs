use std::{
    any::{self, Any},
    fmt::Debug,
};

/// Trait for adding debug to `Box<dyn Any>`. It also adds a method to get the type name of `Self` that's inside the `Box`.
pub trait AnyDebug: Any + Send + Sync + Debug {
    /// Converts `Box<Self>` to `Box<dyn Any + Send + Sync>`
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
