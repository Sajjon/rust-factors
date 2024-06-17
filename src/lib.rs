#![feature(async_closure)]

mod signatures_builders;
mod types;

pub mod prelude {
    pub use crate::signatures_builders::*;
    pub use crate::types::*;

    pub use std::collections::{HashMap, HashSet};

    pub use indexmap::{IndexMap, IndexSet};
}

pub use prelude::*;

#[cfg(test)]
mod tests {
    #[test]
    fn test_() {
        assert_eq!(1, 1);
    }
}
