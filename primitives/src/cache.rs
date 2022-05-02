
pub trait Cache<AccountId, I> {
    fn insert(id: AccountId, data: I);
    fn get(id: AccountId) -> Option<I>;
}
