pub trait Membership<AccountId> {
	fn is_registered(sender: &AccountId) -> bool;
}

impl<AccountId> Membership<AccountId> for () {
	fn is_registered(_sender: &AccountId) -> bool {
		Default::default()
	}
}