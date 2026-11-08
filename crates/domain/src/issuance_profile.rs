use std::time;

use crate::{
    policy::SubjectPolicy,
    uuid::{FamilyId, IssuanceProfileId},
};

pub struct IssuanceProfile {
    pub id: IssuanceProfileId,
    pub family_id: FamilyId,

    pub name: String,
    pub validity: time::Duration,

    pub key_usages: Vec<KeyUsage>,
    pub extended_key_usages: Vec<ExtendedKeyUsage>,

    pub subject_policy: SubjectPolicy,
    pub san_policy: SanPolicy,
}

/// Key usages as defined in [RFC 5280 section 4.2.1.3](https://www.rfc-editor.org/info/rfc5280/#section-4.2.1.3)
pub enum KeyUsage {
    DigitalSignature,
    NonRepudiation,
    KeyEncipherment,
    DataEncipherment,
    KeyAgreement,
    KeyCertSign,
    CrlSign,
    EncipherOnly,
    DecipherOnly,
}

/// Extended key usages as defined in [RFC 5280 section 4.2.1.12](https://www.rfc-editor.org/info/rfc5280/#section-4.2.1.12)
pub enum ExtendedKeyUsage {
    ServerAuth,
    ClientAuth,
    CodeSigning,
    EmailProtection,
    TimeStamping,
    OCSPSigning,
}

pub struct SanPolicy {
    pub dns_names: Vec<String>,
    pub ip_addresses: Vec<String>,
    pub email_addresses: Vec<String>,
    pub uris: Vec<String>,
}
