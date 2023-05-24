use std::{
    collections::HashMap,
    fmt::Debug,
    net::ToSocketAddrs,
    sync::{
        mpsc::{self, Sender, SyncSender},
        Arc, RwLock,
    },
    thread::JoinHandle,
};

use rasta_rs::{
    message::RastaId,
    sci::{
        scip::{SCIPointLocation, SCIPointTargetLocation},
        SCICommand, SCIConnection, SCIMessageType, SCITelegram,
    },
    RastaConnection,
};

pub(crate) use crate::point::PointState;
use crate::{point::GenericPoint, TrackElement, TrackElementError};

impl From<SCIPointLocation> for PointState {
    fn from(value: SCIPointLocation) -> Self {
        match value {
            SCIPointLocation::PointLocationRight => PointState::Right,
            SCIPointLocation::PointLocationLeft => PointState::Left,
            SCIPointLocation::PointNoTargetLocation => unimplemented!(),
            SCIPointLocation::PointBumped => unimplemented!(),
        }
    }
}

impl From<PointState> for SCIPointTargetLocation {
    fn from(value: PointState) -> Self {
        match value {
            PointState::Left => SCIPointTargetLocation::PointLocationChangeToLeft,
            PointState::Right => SCIPointTargetLocation::PointLocationChangeToRight,
        }
    }
}

pub struct SCIPoint {
    state: Arc<RwLock<PointState>>,
    handle: JoinHandle<()>,
    transmitter: SyncSender<PointState>,
    id: String,
}

impl Debug for SCIPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SCIPoint")
            .field("state", &self.state)
            .field("handle", &self.handle)
            .field("transmitter", &self.transmitter)
            .field("id", &self.id)
            .finish()
    }
}

impl SCIPoint {
    pub fn new<A: ToSocketAddrs>(
        state: PointState,
        id: String,
        addr: A,
        rasta_id: RastaId,
        scip_name: String,
        peer: String,
        sci_name_rasta_id_mapping: HashMap<String, RastaId>,
    ) -> Self {
        let state = Arc::new(RwLock::new(state));
        let thread_state = Arc::clone(&state);
        let (transmitter, receiver) = mpsc::sync_channel::<PointState>(256);
        let conn = RastaConnection::try_new(addr, rasta_id).unwrap();
        let id_clone = id.clone();
        let mut scip_conn =
            SCIConnection::try_new(conn, scip_name, sci_name_rasta_id_mapping).unwrap();
        let handle = std::thread::spawn(move || {
            scip_conn
                .run(&peer, |telegram| {
                    if let Some(telegram) = telegram {
                        if telegram.message_type == SCIMessageType::scip_location_status() {
                            let new_state: PointState =
                                SCIPointLocation::try_from(telegram.payload.data[0])
                                    .unwrap()
                                    .into();

                            *thread_state.write().unwrap() = new_state;
                            println!("Point {id_clone} is now {new_state:?}");
                        }
                    }

                    match receiver.try_recv() {
                        Ok(new_state) => {
                            println!("Sending change location command");
                            SCICommand::Telegram(SCITelegram::change_location(
                                "C",
                                "S",
                                SCIPointTargetLocation::from(new_state),
                            ))
                        }
                        Err(_) => SCICommand::Wait,
                    }
                })
                .unwrap();
        });
        Self {
            state,
            handle,
            id,
            transmitter,
        }
    }

    pub fn new_arc<A: ToSocketAddrs>(
        state: PointState,
        id: String,
        addr: A,
        rasta_id: RastaId,
        scip_name: String,
        peer: String,
        sci_name_rasta_id_mapping: HashMap<String, RastaId>,
    ) -> Arc<RwLock<Box<dyn GenericPoint + Send + Sync>>> {
        Arc::new(RwLock::new(Box::new(Self::new(
            state,
            id,
            addr,
            rasta_id,
            scip_name,
            peer,
            sci_name_rasta_id_mapping,
        ))))
    }
}

impl TrackElement for SCIPoint {
    type State = PointState;

    fn id(&self) -> &str {
        &self.id
    }

    fn state(&self) -> Self::State {
        *self.state.read().unwrap()
    }

    fn set_state(&mut self, new_state: Self::State) -> Result<(), crate::TrackElementError> {
        self.transmitter
            .send(new_state)
            .map_err(|_| TrackElementError::RastaError)?;

        Ok(())
    }
}

impl GenericPoint for SCIPoint {}
