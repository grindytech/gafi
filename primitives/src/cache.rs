
pub trait Cache<AccountId, I> {
    fn insert(data: I);
    fn get(id: AccountId) -> Option<I>;
}
