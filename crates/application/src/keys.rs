use casrs_domain::key_reference::{KeyAlgorithm, KeyReference};

pub enum KeyError {
    GenerationFailed(String),
}

#[async_trait::async_trait]
pub trait KeyProvider: Send + Sync {
    async fn generate(&self, algorithm: KeyAlgorithm) -> Result<KeyReference, KeyError>;

    async fn public_key_der(&self, key_reference: &KeyReference) -> Result<Vec<u8>, KeyError>;

    async fn sign(&self, key: &KeyReference, request: SignRequest<'_>)
    -> Result<Vec<u8>, KeyError>;
}

pub struct SignRequest<'a> {
    pub algorithm: SignatureAlgorithm,
    pub message: &'a [u8],
}
