pub trait StreamCard {
    fn observe(&mut self, obs: u64);

    fn card(&self) -> u64;
}
