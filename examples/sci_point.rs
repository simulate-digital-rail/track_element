use std::collections::HashMap;

use std::net::SocketAddr;
use track_element::driveway::DrivewayState;
use track_element::signal::{MainSignalState, SupportedSignalStates};
use track_element::{
    driveway::Driveway,
    point::PointState,
    sci_point::SCIPoint,
    signal::{Signal, SignalState},
};

fn main() {
    let addr: SocketAddr = "127.0.0.1:8888".parse().unwrap();
    let sci_name_rasta_id_mapping =
        HashMap::from([("P1".to_string(), 42), ("S".to_string(), 1337)]);
    let p1 = SCIPoint::new_arc(
        PointState::Left,
        "P1".to_string(),
        addr,
        42,
        "P1".to_string(),
        "S".to_string(),
        sci_name_rasta_id_mapping,
    );
    let s = Signal::new_arc(
        SignalState::default(),
        SupportedSignalStates::default()
            .main(&mut vec![MainSignalState::Hp0, MainSignalState::Ks1]),
        "S".to_string(),
        None,
    );

    let ts = DrivewayState::new(
        vec![(p1.clone(), PointState::Right)],
        vec![(s.clone(), (MainSignalState::Ks1).into())],
        vec![],
    );

    let mut dw = Driveway::new(Vec::new(), ts, s.clone(), s.clone());
    std::thread::sleep(std::time::Duration::from_secs(1));
    dw.set_way().unwrap();

    std::thread::sleep(std::time::Duration::from_secs(1));

    assert!(matches!(p1.read().unwrap().state(), PointState::Right));
    assert!(matches!(
        s.read().unwrap().state().main(),
        MainSignalState::Ks1
    ));
}
