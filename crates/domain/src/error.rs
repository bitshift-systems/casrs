pub enum DomainError {
    /// The authority is disabled and cannot perform the requested operation.
    AuthorityDisabled,

    /// This authority's policy prevents the provisioning of intermediate CAs.
    IntermediateNotAllowed,
}
