use crate::{signal::SignalState, TrackElement};

mod sci_message;

mod sci {
    tonic::include_proto!("sci");
}

pub struct GrpcSignal {}

impl TrackElement for GrpcSignal {
    type State = SignalState;

    fn id(&self) -> &str {
        todo!()
    }

    fn state(&self) -> Self::State {
        todo!()
    }

    fn set_state(&mut self, new_state: Self::State) -> Result<(), crate::TrackElementError> {
        todo!()
    }
}
