//! Domain modeling for Certificates

use crate::uuid::{AuthorityVersionId, CertificateId, FamilyId};

/// A CA certificate issued by an [Authority](crate::authority::Authority).
pub struct CaCertificate {
    pub id: CertificateId,

    /// The CA key represented by this certificate.
    pub subject_version_id: AuthorityVersionId,

    /// The CA key that signed this certificate.
    pub issuer_version_id: AuthorityVersionId,

    /// The certificate in DER format.
    pub der: Vec<u8>,

    /// The serial number of the certificate.
    pub serial_number: Vec<u8>,

    pub not_before: std::time::SystemTime,
    pub not_after: std::time::SystemTime,
}

/// A certificate issued by a [CA](CaCertificate) to an end-entity.
pub struct IssuedCertificate {
    pub id: CertificateId,

    pub family_id: FamilyId,

    /// The CA key that signed this certificate.
    pub issuer_version_id: AuthorityVersionId,

    /// The certificate in DER format.
    pub der: Vec<u8>,

    /// The serial number of the certificate.
    pub serial_number: Vec<u8>,

    pub not_before: std::time::SystemTime,
    pub not_after: std::time::SystemTime,
}
