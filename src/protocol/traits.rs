//! Common traits.

/// Since [`crate::protocol`] entities are immutable by design (after all, they just represent XML
/// definitions), we use
/// [builder](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html) pattern
/// instead of constructor.
pub trait Builder {
    /// Entity with is subjected to `builder` pattern.
    type Buildable: Buildable;

    /// Create an instance of builder.
    ///
    /// An implementation of [`Builder`] returned by this function is initialized
    /// with default values. Once desired values are set, you can call [`Builder::build`] to
    /// obtain [`Buildable`].
    fn build(&self) -> Self::Buildable;
}

/// Subject of the `builder` pattern.
///
/// See: [`Builder`].
pub trait Buildable {
    /// Builder for this entity.
    type Builder: Builder;

    /// Instantiates builder initialised with current values.
    fn to_builder(&self) -> Self::Builder;
}
