use std::time;

use crate::uuid::FamilyId;

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
