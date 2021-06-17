//! Chain Vote

#[macro_use]
mod macros;
pub mod committee;
mod cryptography;
mod encrypted_vote;
mod gang;
mod math;
pub mod tally;

// re-export under a debug module
#[doc(hidden)]
pub mod debug {
    pub mod gang {
        pub use crate::gang::*;
    }
    // pub mod cryptography {
    //     pub use crate::cryptography::*;
    // }
}
