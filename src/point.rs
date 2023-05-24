use std::sync::{Arc, RwLock};

use crate::{TrackElement, TrackElementError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PointState {
    #[default]
    Left,
    Right,
}

#[derive(Debug)]
pub struct Point {
    state: PointState,
    id: String,
}

impl Point {
    pub fn new(state: PointState, id: String) -> Self {
        Self { state, id }
    }

    pub fn new_arc(
        state: PointState,
        id: String,
    ) -> Arc<RwLock<Box<dyn GenericPoint + Send + Sync>>> {
        Arc::new(RwLock::new(Box::new(Self::new(state, id))))
    }
}

pub trait GenericPoint: TrackElement<State = PointState> {}

impl GenericPoint for Point {}

impl TrackElement for Point {
    type State = PointState;

    fn id(&self) -> &str {
        &self.id
    }

    fn state(&self) -> Self::State {
        self.state
    }

    fn set_state(&mut self, new_state: Self::State) -> Result<(), TrackElementError> {
        self.state = new_state;
        println!("Point state is now {:?}", self.state);
        Ok(())
    }
}
