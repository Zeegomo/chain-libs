#[macro_use]
extern crate cfg_if;

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;
extern crate hex;
extern crate rand_core;

cfg_if! {
    if #[cfg(test)] {
        mod testing;
    } else if #[cfg(feature = "property-test-api")] {
        pub mod testing;
    }
}

pub mod algorithms;
pub mod asymlock;
pub mod bech32;
pub mod digest;
mod evolving;
pub mod hash;
mod kes;
mod key;
pub mod multilock;
mod sign;
mod vrf;

pub mod role;

pub use evolving::{EvolvingStatus, KeyEvolvingAlgorithm};
pub use kes::KeyEvolvingSignatureAlgorithm;
pub use key::{
    AsymmetricKey, AsymmetricPublicKey, KeyPair, PublicKey, PublicKeyError, PublicKeyFromStrError,
    SecretKey, SecretKeyError, SecretKeySizeStatic,
};
pub use sign::{
    Signature, SignatureError, SignatureFromStrError, SigningAlgorithm, Verification,
    VerificationAlgorithm,
};
pub use vrf::{
    vrf_evaluate_and_prove, vrf_verified_get_output, vrf_verify, VerifiableRandomFunction,
    VrfVerification,
};

pub use algorithms::*;
pub use hash::Blake2b256;
