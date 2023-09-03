use core::fmt;

pub(crate) struct DebugFn<F>(pub(crate) F)
where
    F: Fn(&mut fmt::Formatter) -> fmt::Result;

impl<F> fmt::Debug for DebugFn<F>
where
    F: Fn(&mut fmt::Formatter) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (self.0)(f)
    }
}
