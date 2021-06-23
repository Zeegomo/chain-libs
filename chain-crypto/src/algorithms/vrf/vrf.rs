//! Verifiable Random Function (VRF) implementation
//! using the 2-Hash-DH verifiable oblivious PRF
//! defined in the Ouroboros Praos paper

use crate::ec::{GroupElement, Scalar};
use crate::hash::Blake2b256;
use rand_core::{CryptoRng, RngCore};
use std::hash::{Hash, Hasher};

use super::dleq;
use crate::key::PublicKeyError;

/// VRF Secret Key
#[derive(Clone)]
pub struct SecretKey {
    secret: Scalar,
    public: GroupElement,
}

impl AsRef<[u8]> for SecretKey {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

/// VRF Public Key
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKey(GroupElement, [u8; PUBLIC_SIZE]);

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for PublicKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state)
    }
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

/// VRF Output (Point)
///
/// This is used to create an output generator tweaked by the VRF.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputSeed(GroupElement);

/// VRF Proof of generation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvenOutputSeed {
    pub(crate) u: OutputSeed,
    dleq_proof: dleq::Proof,
}

pub const PROOF_SIZE: usize = dleq::Proof::PROOF_SIZE + GroupElement::BYTES_LEN;
pub const SECRET_SIZE: usize = Scalar::BYTES_LEN;
pub const PUBLIC_SIZE: usize = GroupElement::BYTES_LEN;

impl SecretKey {
    /// Create a new random secret key
    pub fn random<T: RngCore + CryptoRng>(mut rng: T) -> Self {
        let sk = Scalar::random(&mut rng);
        let pk = GroupElement::generator() * sk;
        SecretKey {
            secret: sk,
            public: pk,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.secret.as_bytes()
    }

    /// Serialize the secret key in binary form
    pub fn to_bytes(&self) -> [u8; SECRET_SIZE] {
        let mut v = [0u8; SECRET_SIZE];
        v.copy_from_slice(&self.secret.to_bytes());
        v
    }

    pub fn from_bytes(bytes: [u8; SECRET_SIZE]) -> Option<Self> {
        let sk = Scalar::from_canonical_bytes(bytes)?;
        let pk = GroupElement::generator() * sk;
        Some(SecretKey {
            secret: sk,
            public: pk,
        })
    }

    /// Get the verifiable output and the associated input base point.
    ///
    /// The following property hold between the return values:
    ///     Point * secret = OutputSeed
    pub fn verifiable_output(&self, input: &[u8]) -> (GroupElement, OutputSeed) {
        let m_point = GroupElement::from_hash(input);
        let u = m_point * self.secret;
        (m_point, OutputSeed(u))
    }

    /// Create a proof, for the given parameters; no check is made to make sure it's correct
    ///
    /// the proof is randomized, so need a freshly randomly scalar for random.
    /// use 'proove_simple' to use a RNG and avoid generating this random.
    ///
    /// use 'evaluate' or 'evaluate_simple' for creating the proof directly from input
    pub fn proove(
        &self,
        r: &Scalar,
        m_point: GroupElement,
        output: OutputSeed,
    ) -> ProvenOutputSeed {
        let dleq = dleq::Dleq {
            g1: &GroupElement::generator(),
            h1: &self.public,
            g2: &m_point,
            h2: &output.0,
        };
        let dleq_proof = dleq::generate(&r, &self.secret, &dleq);
        ProvenOutputSeed {
            u: output.clone(),
            dleq_proof,
        }
    }

    pub fn proove_simple<T: RngCore + CryptoRng>(
        &self,
        rng: &mut T,
        m_point: GroupElement,
        output: OutputSeed,
    ) -> ProvenOutputSeed {
        let w = Scalar::random(rng);
        self.proove(&w, m_point, output)
    }

    /// Generate a Proof
    ///
    /// the proof is randomized, so need a freshly randomly scalar for random.
    /// use 'evaluate_simple' for normal use.
    pub fn evaluate(&self, r: &Scalar, input: &[u8]) -> ProvenOutputSeed {
        let (m_point, output) = self.verifiable_output(input);
        self.proove(r, m_point, output)
    }

    pub fn evaluate_simple<T: RngCore + CryptoRng>(
        &self,
        rng: &mut T,
        input: &[u8],
    ) -> ProvenOutputSeed {
        let (m_point, output) = self.verifiable_output(input);
        self.proove_simple(rng, m_point, output)
    }

    /// Get the public key associated with a secret key
    pub fn public(&self) -> PublicKey {
        PublicKey(self.public, self.public.to_bytes())
    }
}

impl PublicKey {
    pub fn from_bytes(input: &[u8]) -> Result<Self, PublicKeyError> {
        if input.len() != PUBLIC_SIZE {
            return Err(PublicKeyError::SizeInvalid);
        }
        let group_element = GroupElement::from_bytes(input);
        match group_element {
            None => Err(PublicKeyError::StructureInvalid),
            Some(pk) => Ok(PublicKey(pk, pk.to_bytes())),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.1
    }

    pub fn to_buffer(&self, output: &mut [u8]) {
        assert_eq!(output.len(), PUBLIC_SIZE);
        output.copy_from_slice(&self.0.to_bytes())
    }
}

impl ProvenOutputSeed {
    /// Verify a proof for a given public key and a data slice
    pub fn verify(&self, public_key: &PublicKey, input: &[u8]) -> bool {
        let dleq = dleq::Dleq {
            g1: &GroupElement::generator(),
            h1: &public_key.0,
            g2: &GroupElement::from_hash(&input),
            h2: &self.u.0,
        };
        dleq::verify(&dleq, &self.dleq_proof)
    }

    pub fn to_buffer(&self, output: &mut [u8]) {
        assert_eq!(output.len(), PROOF_SIZE);
        output[0..32].copy_from_slice(&self.u.0.to_bytes());
        self.dleq_proof.to_bytes(&mut output[32..96]);
    }

    pub fn bytes(&self) -> [u8; PROOF_SIZE] {
        let mut output = [0u8; PROOF_SIZE];
        output[0..32].copy_from_slice(&self.u.0.to_bytes());
        self.dleq_proof.to_bytes(&mut output[32..96]);
        output
    }

    pub fn from_bytes_unverified(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != PROOF_SIZE {
            return None;
        }
        let u = GroupElement::from_bytes(&bytes[0..32])?;
        let proof = dleq::Proof::from_bytes(&bytes[32..])?;
        Some(ProvenOutputSeed {
            u: OutputSeed(u),
            dleq_proof: proof,
        })
    }

    pub fn from_bytes(public_key: &PublicKey, bytes: &[u8], input: &[u8]) -> Option<Self> {
        let pos = Self::from_bytes_unverified(bytes)?;
        if pos.verify(public_key, input) {
            Some(pos)
        } else {
            None
        }
    }

    pub fn to_output(&self) -> OutputSeed {
        self.u.clone()
    }

    pub fn to_verifiable_output(&self, public_key: &PublicKey, input: &[u8]) -> Option<OutputSeed> {
        if self.verify(public_key, input) {
            Some(self.u.clone())
        } else {
            None
        }
    }
}

impl OutputSeed {
    /// Get the output for this input and a known suffix
    pub fn to_output(&self, input: &[u8], suffix: &[u8]) -> Blake2b256 {
        let mut buf = Vec::new();
        buf.extend_from_slice(input);
        buf.extend_from_slice(&self.0.to_bytes());
        buf.extend_from_slice(suffix);

        Blake2b256::new(&buf)
    }
}

#[cfg(test)]
mod tests {
    use super::SecretKey;
    use rand_core::{OsRng, RngCore};

    #[test]
    fn it_works() {
        let mut csprng: OsRng = OsRng;
        let sk = SecretKey::random(&mut csprng);
        let pk = sk.public();

        let sk_other = SecretKey::random(&mut csprng);
        let pk_other = sk_other.public();

        let mut b1 = [0u8; 10];
        for i in b1.iter_mut() {
            *i = csprng.next_u32() as u8;
        }
        let mut b2 = [0u8; 10];
        for i in b2.iter_mut() {
            *i = csprng.next_u32() as u8;
        }

        let proof = sk.evaluate_simple(&mut csprng, &b1[..]);

        // make sure the test pass
        assert_eq!(proof.verify(&pk, &b1[..]), true);

        // now try with false positive
        assert_eq!(proof.verify(&pk, &b2[..]), false);
        assert_eq!(proof.verify(&pk_other, &b1[..]), false);
        assert_eq!(proof.verify(&pk_other, &b2[..]), false);
    }
}
