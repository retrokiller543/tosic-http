//! # Route
//!
//! When parsing params we currently use a `BTreeMap`, this is fine until we try extract
//! the parameter in order since the BTreeMap will keep it sorted and possibly not in the same order
//! as what the user might think or see in the path. This is a better solution than using the
//! original solution which was a `HashMap` which failed sometimes, and sometimes it worked based on
//! the hashing algo

pub(crate) mod node;
mod path;

#[allow(clippy::module_inception)]
mod route;
mod test;

pub use node::*;
pub use route::*;
