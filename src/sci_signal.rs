use std::{
    collections::HashMap,
    fmt::Debug,
    net::ToSocketAddrs,
    sync::{
        mpsc::{self, Sender},
        Arc, RwLock,
    },
    thread::JoinHandle,
};

use rasta_rs::{
    message::RastaId,
    sci::{
        scils::{SCILSMain, SCILSSignalAspect},
        SCICommand, SCIConnection, SCIMessageType, SCITelegram,
    },
    RastaConnection,
};

pub(crate) use crate::signal::SignalState;
use crate::{TrackElement, TrackElementError};

impl From<SCILSMain> for SignalState {
    fn from(value: SCILSMain) -> Self {
        match value {
            SCILSMain::Hp0 => SignalState::Hp0,
            SCILSMain::Hp0PlusSh1 => SignalState::Hp0PlusSh1,
            SCILSMain::Hp0WithDrivingIndicator => SignalState::Hp0WithDrivingIndicator,
            SCILSMain::Ks1 => SignalState::Ks1,
            SCILSMain::Ks1Flashing => SignalState::Ks1Flashing,
            SCILSMain::Ks1FlashingWithAdditionalLight => {
                SignalState::Ks1FlashingWithAdditionalLight
            }

            SCILSMain::Ks2 => SignalState::Ks2,
            SCILSMain::Ks2WithAdditionalLight => SignalState::Ks2WithAdditionalLight,
            SCILSMain::Sh1 => SignalState::Sh1,
            SCILSMain::IdLight => SignalState::IdLight,
            SCILSMain::Hp0Hv => SignalState::Hp0Hv,
            SCILSMain::Hp1 => SignalState::Hp1,
            SCILSMain::Hp2 => SignalState::Hp2,
            SCILSMain::Vr0 => SignalState::Vr0,
            SCILSMain::Vr1 => SignalState::Vr1,
            SCILSMain::Vr2 => SignalState::Vr2,
            SCILSMain::Off => SignalState::Off,
        }
    }
}

impl From<SignalState> for SCILSMain {
    fn from(value: SignalState) -> Self {
        match value {
            SignalState::Hp0 => SCILSMain::Hp0,
            SignalState::Hp0PlusSh1 => SCILSMain::Hp0PlusSh1,
            SignalState::Hp0WithDrivingIndicator => SCILSMain::Hp0WithDrivingIndicator,
            SignalState::Ks1 => SCILSMain::Ks1,
            SignalState::Ks1Flashing => SCILSMain::Ks1Flashing,
            SignalState::Ks1FlashingWithAdditionalLight => {
                SCILSMain::Ks1FlashingWithAdditionalLight
            }

            SignalState::Ks2 => SCILSMain::Ks2,
            SignalState::Ks2WithAdditionalLight => SCILSMain::Ks2WithAdditionalLight,
            SignalState::Sh1 => SCILSMain::Sh1,
            SignalState::IdLight => SCILSMain::IdLight,
            SignalState::Hp0Hv => SCILSMain::Hp0Hv,
            SignalState::Hp1 => SCILSMain::Hp1,
            SignalState::Hp2 => SCILSMain::Hp2,
            SignalState::Vr0 => SCILSMain::Vr0,
            SignalState::Vr1 => SCILSMain::Vr1,
            SignalState::Vr2 => SCILSMain::Vr2,
            SignalState::Off => SCILSMain::Off,
        }
    }
}

impl From<SCILSSignalAspect> for SignalState {
    fn from(value: SCILSSignalAspect) -> Self {
        value.main().into()
    }
}

impl From<SignalState> for SCILSSignalAspect {
    fn from(value: SignalState) -> Self {
        SCILSSignalAspect::new(
            value.into(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            [0; 9],
        )
    }
}

pub struct SCISignal {
    state: Arc<RwLock<SignalState>>,
    handle: JoinHandle<()>,
    transmitter: Sender<SignalState>,
    id: String,
}

impl Debug for SCISignal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SCIPoint")
            .field("state", &self.state)
            .field("handle", &self.handle)
            .field("transmitter", &self.transmitter)
            .field("id", &self.id)
            .finish()
    }
}

impl SCISignal {
    pub fn new<A: ToSocketAddrs>(
        state: SignalState,
        id: String,
        addr: A,
        rasta_id: RastaId,
        scip_name: String,
        peer: String,
        sci_name_rasta_id_mapping: HashMap<String, RastaId>,
    ) -> Self {
        let state = Arc::new(RwLock::new(state));
        let thread_state = Arc::clone(&state);
        let (transmitter, receiver) = mpsc::channel::<SignalState>();
        let conn = RastaConnection::try_new(addr, rasta_id).unwrap();
        let id_clone = id.clone();
        let mut scip_conn =
            SCIConnection::try_new(conn, scip_name, sci_name_rasta_id_mapping).unwrap();
        let handle = std::thread::spawn(move || {
            scip_conn
                .run(&peer, |telegram| {
                    if let Some(telegram) = telegram {
                        if telegram.message_type == SCIMessageType::scils_signal_aspect_status() {
                            let new_state: SignalState =
                                SCILSSignalAspect::try_from(telegram.payload.data.as_slice())
                                    .unwrap()
                                    .into();

                            *thread_state.write().unwrap() = new_state;
                            println!("Signal {id_clone} is now {new_state:?}");
                        }
                    }

                    match receiver.try_recv() {
                        Ok(new_state) => {
                            println!("Sending signal aspect change command");
                            SCICommand::Telegram(SCITelegram::scils_show_signal_aspect(
                                "C",
                                "S",
                                SCILSSignalAspect::from(new_state),
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
}

impl TrackElement for SCISignal {
    type State = SignalState;

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
