//! Domain modelling for Authorities

use std::time;

use crate::{
    error::DomainError,
    key_reference::KeyReference,
    uuid::{AuthorityId, AuthorityVersionId, FamilyId},
};

/// A certificate authority (CA) that can issue other certificates.
pub struct Authority {
    pub id: AuthorityId,
    pub family_id: FamilyId,

    pub name: String,
    pub parent_id: Option<AuthorityId>,

    pub active_version_id: Option<AuthorityVersionId>,
    pub status: AuthorityStatus,

    pub policy: AuthorityPolicy,
}

impl Authority {
    pub fn create_intermediate(
        &self,
        id: AuthorityId,
        name: String,
        policy: AuthorityPolicy,
    ) -> Result<Authority, DomainError> {
        if self.status != AuthorityStatus::Enabled {
            return Err(DomainError::AuthorityNotEnabled);
        }

        if !self.policy.can_issue_subordinate_cas {
            return Err(DomainError::IntermediateNotAllowed);
        }

        Ok(Authority {
            id,
            family_id: self.family_id,
            name,
            parent_id: Some(self.id),
            active_version_id: None,
            status: AuthorityStatus::Enabled,
            policy,
        })
    }
}

pub enum AuthorityStatus {
    Enabled,
    Disabled,
    Retired,
}

pub struct AuthorityPolicy {
    pub max_path_length: Option<u32>,
    pub max_certificate_lifetime: time::Duration,

    pub can_issue_subordinate_cas: bool,
    pub can_issue_end_entities: bool,
}

pub struct AuthorityVersion {
    pub id: AuthorityVersionId,
    pub authority_id: AuthorityId,

    pub version: u32,
    pub key: KeyReference,

    pub status: AuthorityVersionStatus,

    pub created_at: time::SystemTime,
    pub activated_at: Option<time::SystemTime>,
}

pub enum AuthorityVersionStatus {
    Pending,
    Ready,
    Active,
    Draining,
    Retired,
    Compromised,
}
