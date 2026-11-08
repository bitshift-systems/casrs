//! Unique identifier types for domain entities.

use uuid::Uuid;

macro_rules! define_id {
    (
        $(#[$meta:meta])*
        $vis: vis $name: ident
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $vis struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }
        }
    }
}

define_id!(
    /// A unique identifier for a [Family](crate::family::Family).
    pub FamilyId
);

define_id!(
    /// A unique identifier for an [Authority](crate::authority::Authority).
    pub AuthorityId
);

define_id!(
    /// A unique identifier for an [AuthorityVersion](crate::authority::AuthorityVersion).
    pub AuthorityVersionId
);

define_id!(
    /// A unique identifier for a [Certificate](crate::certificate::Certificate).
    pub CertificateId
);

define_id!(
    /// A unique identifier for a [IssuanceProfile](crate::issuance_profile::IssuanceProfile).
    pub IssuanceProfileId
);
