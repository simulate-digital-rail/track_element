use signal::MainSignalState;

pub mod control_station;
pub mod driveway;
pub mod point;
pub mod signal;

#[cfg(feature = "sci_point")]
pub mod sci_point;

#[cfg(feature = "sci_signal")]
pub mod sci_signal;

#[cfg(test)]
mod test;
pub mod vacancy_section;

#[derive(Debug)]
pub enum TrackElementError {
    DrivewayDoesNotExist(String),
    HasConflictingDriveways,
    InvalidAdditionalSignalState,
    InvalidMainSignalState(MainSignalState),
    RastaError,
}

impl std::fmt::Display for TrackElementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for TrackElementError {}

pub trait TrackElement: std::fmt::Debug {
    type State: Copy + Default;

    fn id(&self) -> &str;
    fn state(&self) -> Self::State;
    fn set_state(&mut self, new_state: Self::State) -> Result<(), TrackElementError>;

    fn reset(&mut self) -> Result<(), TrackElementError> {
        self.set_state(Self::State::default())
    }
}
