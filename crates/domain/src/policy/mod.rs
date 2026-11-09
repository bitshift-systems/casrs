mod san;
mod subject;

pub use san::*;
pub use subject::*;

/// A certificate identity, consisting of a subject and a set of Subject
/// Alternative Names (SANs).
#[derive(Debug, Clone)]
pub struct CertificateIdentity {
    /// The subject of the certificate, represented as a [DistinguishedName].
    pub subject: DistinguishedName,
    /// The Subject Alternative Names (SANs) of the certificate, represented as
    /// a set of [SubjectAltName]s.
    pub subject_alt_names: Vec<SubjectAltName>,
}

#[derive(Debug, thiserror::Error)]
pub enum CertificateIdentityError {
    #[error(transparent)]
    SubjectValidationError(#[from] SubjectPolicyError),
    #[error(transparent)]
    SanValidationError(#[from] SanPolicyError),
}

impl CertificateIdentity {
    /// Creates a new [CertificateIdentity] with the given subject and SANs.
    pub fn new(subject: DistinguishedName, subject_alt_names: Vec<SubjectAltName>) -> Self {
        Self {
            subject,
            subject_alt_names,
        }
    }

    /// Validates the certificate identity against the given subject and SAN
    /// policies.
    ///
    /// See [SubjectPolicy] and [SanPolicy] for details on the validation rules.
    pub fn validate(
        &self,
        subject_policy: &SubjectPolicy,
        san_policy: &SanPolicy,
    ) -> Result<(), CertificateIdentityError> {
        subject_policy.validate(&self.subject)?;
        for san in &self.subject_alt_names {
            san_policy.validate(san)?;
        }
        Ok(())
    }
}
