//! Formatting utils

use std::fmt;

/// Format the iterator like a map
pub struct DebugMap<F>(pub F);

impl<'a, F, I, K, V> fmt::Debug for DebugMap<F>
    where F: Fn() -> I,
          I: IntoIterator<Item=(K, V)>,
          K: fmt::Debug,
          V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map()
         .entries((self.0)())
         .finish()
    }
}

/// Avoid "pretty" debug
pub struct NoPretty<T>(pub T);

impl<T> fmt::Debug for NoPretty<T>
    where T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

