pub enum RepositoryError {
    NotFound,
    DatabaseError(String),
    Other(String),
}
