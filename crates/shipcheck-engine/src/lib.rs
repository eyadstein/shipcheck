//! Rule loading and project scanning for Shipcheck.

mod error;
mod rule;
mod scanner;
mod walk;

pub use error::{Error, Result};
pub use rule::{Catalog, Compiled, Matcher, Rule};
pub use scanner::scan;
pub use walk::collect_files;
