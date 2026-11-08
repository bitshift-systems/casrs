mod san;
mod subject;

pub use san::*;
pub use subject::*;

#[derive(Debug, Clone)]
pub struct CertificateIdentity {
    pub subject: DistinguishedName,
    pub subject_alt_names: Vec<SubjectAltName>,
}
