use casrs_domain::{
    authority::{Authority, AuthorityVersion},
    uuid::{AuthorityId, AuthorityVersionId},
};

use crate::error::RepositoryError;

#[async_trait::async_trait]
pub trait AuthorityRepository: Send + Sync {
    async fn get(&self, id: AuthorityId) -> Result<Option<Authority>, RepositoryError>;

    async fn get_versiohn(
        &self,
        authority_id: AuthorityVersionId,
    ) -> Result<Option<AuthorityVersion>, RepositoryError>;

    async fn list_children(
        &self,
        parent_id: AuthorityId,
    ) -> Result<Vec<Authority>, RepositoryError>;
}
