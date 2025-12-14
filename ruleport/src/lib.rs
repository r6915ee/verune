pub mod de;
pub mod error;
pub mod ser;

use std::fmt::{self, Display, Formatter};

// pub use de::{Deserializer, from_str};
pub use error::{Error, Result};
// pub use ser::{Serializer, to_string};

#[derive(Debug)]
pub enum Iterable {
    Tuple,
    Array,
}

impl Display for Iterable {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Iterable::Tuple => formatter.write_str("tuple"),
            Iterable::Array => formatter.write_str("array"),
        }
    }
}
