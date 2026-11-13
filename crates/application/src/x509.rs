use casrs_domain::{
    issuance_profile::{ExtendedKeyUsage, KeyUsage},
    key_reference::KeyReference,
};

pub enum CertificateError {
    CreationFailed(String),
}

#[async_trait::async_trait]
pub trait CertificateEngine: Send + Sync {
    async fn create_root(
        &self,
        spec: RootCertificateSpec,
        key: &KeyReference,
    ) -> Result<Vec<u8>, CertificateError>;

    async fn create_intermediate(
        &self,
        spec: IntermediateCertificateSpec,
        subject_key: &KeyReference,
        issuer: &IssuerContext,
    ) -> Result<SignedCertificate, CertificateError>;

    async fn issue(
        &self,
        spec: EndEntityCertificateSpec,
        issuer: &IssuerContext,
    ) -> Result<SignedCertificate, CertificateError>;
}

pub struct RootCertificateSpec {
    pub subject: String,
    pub validity: std::time::Duration,
    pub key_usages: Vec<KeyUsage>,
    pub extended_key_usages: Vec<ExtendedKeyUsage>,
}

pub struct IntermediateCertificateSpec {
    pub subject: String,
    pub validity: std::time::Duration,
    pub key_usages: Vec<KeyUsage>,
    pub extended_key_usages: Vec<ExtendedKeyUsage>,
}

pub struct EndEntityCertificateSpec {
    pub subject: String,
    pub validity: std::time::Duration,
    pub key_usages: Vec<KeyUsage>,
    pub extended_key_usages: Vec<ExtendedKeyUsage>,
}

pub struct IssuerContext {
    pub issuer_certificate: Vec<u8>,
    pub issuer_key: KeyReference,
}

#[derive(Debug, Clone)]
pub struct SignedCertificate {
    /// Complete DER-encoded X.509 certificate.
    pub der: Vec<u8>,
}

pub struct CertificateBundle {
    pub certificate: SignedCertificate,
    /// Issuing intermediates in leaf-to-root order.
    /// Does not include the self-signed trust anchor.
    pub issuer_chain: Vec<SignedCertificate>,
}
