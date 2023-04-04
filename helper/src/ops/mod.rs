pub trait Tap: Sized {
    #[must_use]
    fn tap<A>(self, f: impl FnOnce(&Self) -> A) -> Self;
}

impl<T: Sized> Tap for T {
    fn tap<A>(self, f: impl FnOnce(&Self) -> A) -> Self {
        let _ = f(&self);
        self
    }
}
