use std::{
    any::{self, Any},
    fmt::Debug,
};

/// Trait for adding debug and some methods to `dyn Any`.
pub trait AnyDebug: Any + Debug + Send + Sync {
    /// Converts `Box<Self>` to `Box<dyn Any + Send + Sync>`.
    fn into_box_any(self: Box<Self>) -> Box<dyn Any + Send + Sync>;
    /// Returns the type name of `Self`.
    ///
    /// For `Box<dyn AnyDebug>` it returns `Box<dyn AnyDebug>` not what's inside the box.
    /// If you need the type name of what's inside the box then you first need to dereference it.
    ///
    /// Example usage:
    /// ```
    /// use orchestrator::generic::AnyDebug;
    ///
    /// let boxed_str: Box<dyn AnyDebug> = Box::new("Hello");
    /// // returns the type name of the box
    /// let box_name = boxed_str.get_type_name();
    /// // returns the type name of what's inside the box
    /// let str_name = (*boxed_str).get_type_name();
    /// assert_eq!(
    ///     "alloc::boxed::Box<dyn orchestrator::generic::any_debug::AnyDebug>",
    ///     box_name
    /// );
    /// assert_eq!("&str", str_name);
    /// ```
    fn get_type_name(&self) -> &'static str;
}

impl<T> AnyDebug for T
where
    T: Any + Debug + Send + Sync,
{
    fn into_box_any(self: Box<Self>) -> Box<dyn Any + Send + Sync> {
        self
    }

    fn get_type_name(&self) -> &'static str {
        any::type_name::<Self>()
    }
}

/// Trait for converting `&dyn Trait` to `&dyn Any`.
///
/// For some usage look at implementation of [`PipelineStorage`](crate::generic::pipeline::PipelineStorage).
pub(crate) trait ToRefAny {
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

pub(crate) trait SuperAnyDebug: AnyDebug + ToRefAny + Send + Sync {
    #[cfg_attr(not(feature = "pipeline_early_return"), allow(dead_code))]
    fn into_box_anydebug(self: Box<Self>) -> Box<dyn AnyDebug>;
}

impl<T> SuperAnyDebug for T
where
    T: AnyDebug + ToRefAny + Send + Sync,
{
    fn into_box_anydebug(self: Box<Self>) -> Box<dyn AnyDebug> {
        self
    }
}
