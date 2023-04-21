
impl<T: Config<T>, I: 'static> Pallet<T, I> {
    pub fn gen_id() -> Result<ID, Error<T>> {
        let payload = (
            T::Randomness::random(&b""[..]).0,
            <frame_system::Pallet<T>>::block_number(),
        );
        Ok(payload.using_encoded(blake2_256))
    }
}