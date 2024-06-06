pub enum DataError {
    ConvertOutOfRange { desc: &'static str },
}

pub enum GuestError {
    EnergyNotEnough {
        energy_reserve: u64,
        energy_required: u64,
        operation: &'static str,
    },
}

pub enum PlayerError{

}
