/// Utility functions and traits for the web crate
pub mod option {
    /// Extension trait for Option that adds the is_none_or method
    pub trait OptionExt<T> {
        /// Returns true if the option is None or the predicate returns true for the contained value
        fn is_none_or<F>(&self, f: F) -> bool
        where
            F: FnOnce(&T) -> bool;
    }

    impl<T> OptionExt<T> for Option<T> {
        fn is_none_or<F>(&self, f: F) -> bool
        where
            F: FnOnce(&T) -> bool,
        {
            match self {
                None => true,
                Some(val) => f(val),
            }
        }
    }
} 