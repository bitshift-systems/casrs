use casrs_domain::key_reference::KeyReference;

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
