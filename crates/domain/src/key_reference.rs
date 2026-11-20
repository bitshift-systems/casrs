//! Domain modelling for Key References

/// A reference to a cryptographic key, which can be used for signing or
/// encryption.
///
/// The key reference contains an identifier, a source (e.g., cleartext), and a
/// key ID that can be used to retrieve the actual key material from a secure
/// storage or key management system.
pub struct KeyReference {
    pub provider: String,
    pub key_id: String,
    pub algorithm: KeyAlgorithm,
}

/// A cryptographic algorithm that can be used for signing or encryption.
pub enum KeyAlgorithm {
    Rsa2048,
    Rsa3072,
    Rsa4096,
    EcdsaP256,
    EcdsaP384,
    EcdsaP521,
    Ed25519,
}
