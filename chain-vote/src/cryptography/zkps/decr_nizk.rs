//! Non-interactive Zero Knowledge proof for correct ElGamal
//! decryption. We use the notation and scheme presented in
//! Figure 5 of the Treasury voting protocol spec.
//!
//! The proof is the following:
//!
//! `NIZK{(pk, C, M), (sk): M = Dec_sk(C) AND pk = g^sk}`
//!
//! which makes the statement, the public key, `pk`, the ciphertext
//! `(e1, e2)`, and the message, `m`. The witness, on the other hand
//! is the secret key, `sk`.
#![allow(clippy::many_single_char_names)]
use crate::cryptography::{Ciphertext, PublicKey, SecretKey};
use crate::gang::{GroupElement, Scalar};
use super::challenge_context::ChallengeContextProofDecrypt;
use rand::{CryptoRng, RngCore};

/// Proof of correct decryption.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProofDecrypt {
    a1: GroupElement,
    a2: GroupElement,
    z: Scalar,
}

impl ProofDecrypt {
    pub(crate) const PROOF_SIZE: usize = 2 * GroupElement::BYTES_LEN + Scalar::BYTES_LEN; // Scalar is 32 bytes
    /// Generate a decryption zero knowledge proof
    pub fn generate<R>(c: &Ciphertext, pk: &PublicKey, sk: &SecretKey, rng: &mut R) -> Self
    where
        R: CryptoRng + RngCore,
    {
        let w = Scalar::random(rng);
        let a1 = GroupElement::generator() * &w;
        let a2 = &c.e1 * &w;
        let d = &c.e1 * &sk.sk;
        let mut challenge = ChallengeContextProofDecrypt::new(pk, c, &d);
        let e = challenge.first_challenge(&a1, &a2);
        let z = &sk.sk * &e + &w;

        ProofDecrypt { a1, a2, z }
    }

    /// Verify a decryption zero knowledge proof
    pub fn verify(&self, c: &Ciphertext, m: &GroupElement, pk: &PublicKey) -> bool {
        let d = &c.e2 - m;
        let mut challenge = ChallengeContextProofDecrypt::new(pk, c, &d);
        let e = challenge.first_challenge(&self.a1, &self.a2);
        let gz = GroupElement::generator() * &self.z;
        let he = &pk.pk * &e;
        let he_a1 = he + &self.a1;
        let c1z = &c.e1 * &self.z;
        let de = d * &e;
        let de_a2 = de + &self.a2;
        gz == he_a1 && c1z == de_a2
    }

    pub fn to_bytes(&self) -> [u8; Self::PROOF_SIZE] {
        let mut output = [0u8; Self::PROOF_SIZE];
        self.to_slice_mut(&mut output);
        output
    }

    pub fn to_slice_mut(&self, output: &mut [u8]) {
        assert_eq!(output.len(), Self::PROOF_SIZE);
        output[0..GroupElement::BYTES_LEN].copy_from_slice(&self.a1.to_bytes());
        output[GroupElement::BYTES_LEN..(2 * GroupElement::BYTES_LEN)]
            .copy_from_slice(&self.a2.to_bytes());
        output[(2 * GroupElement::BYTES_LEN)..(2 * GroupElement::BYTES_LEN) + Scalar::BYTES_LEN]
            .copy_from_slice(&self.z.to_bytes());
    }

    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() != Self::PROOF_SIZE {
            return None;
        }
        let a1 = GroupElement::from_bytes(&slice[0..GroupElement::BYTES_LEN])?;
        let a2 = GroupElement::from_bytes(
            &slice[GroupElement::BYTES_LEN..(2 * GroupElement::BYTES_LEN)],
        )?;
        let z = Scalar::from_bytes(
            &slice
                [(2 * GroupElement::BYTES_LEN)..(2 * GroupElement::BYTES_LEN) + Scalar::BYTES_LEN],
        )?;

        let proof = ProofDecrypt { a1, a2, z };
        Some(proof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cryptography::Keypair;
    use rand_chacha::ChaCha20Rng;
    use rand_core::SeedableRng;

    #[test]
    pub fn it_works() {
        let mut r = ChaCha20Rng::from_seed([0u8; 32]);

        let keypair = Keypair::generate(&mut r);

        let plaintext = GroupElement::from_hash(&[0u8]);
        let ciphertext = keypair.public_key.encrypt_point(&plaintext, &mut r);

        let proof = ProofDecrypt::generate(
            &ciphertext,
            &keypair.public_key,
            &keypair.secret_key,
            &mut r,
        );
        let verified = proof.verify(&ciphertext, &plaintext, &keypair.public_key);
        assert_eq!(verified, true);
    }
}
