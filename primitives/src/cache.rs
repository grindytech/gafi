
pub trait Cache<AccountId, A, D> {
    fn insert(id: &AccountId, action: A, data: D);
    fn get(id: &AccountId, action: A) -> Option<D>;
}
