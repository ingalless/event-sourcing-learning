use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::Write,
    time::{Instant, SystemTime},
};

#[derive(Serialize, Deserialize, Debug)]
struct Event {
    id: String,

    /// UTC Timestamp of the event
    ts: u64,
    event: StateEvent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
enum StateEvent {
    /// Adds a new ship to the oceans
    EnrolShip { ship: String },

    /// The ship has arrived at a port
    Arrival { ship: String, port: Port },

    /// The ship has left a port and is out at sea
    Departure { ship: String },
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Port {
    SanFrancisco,
    Porto,
    LosAngeles,
    HongKong,
    Tokyo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Ship {
    name: String,
    port: Option<Port>,
}

impl Ship {
    fn new(name: String, port: Option<Port>) -> Ship {
        Ship { name, port }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct State {
    ships: HashMap<String, Ship>,
}

fn process_event(event: StateEvent, state: &mut State) {
    log_event(Event {
        id: "a".into(),
        ts: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("UTC timestamp should be after UNIX_EPOCH")
            .as_secs(),
        event: event.clone(),
    });
    match event {
        StateEvent::EnrolShip { ship } => {
            state.ships.insert(ship.clone(), Ship::new(ship, None));
        }
        StateEvent::Arrival { ship, port } => {
            state
                .ships
                .entry(ship)
                .and_modify(|s| s.port = Some(port.clone()));
        }
        StateEvent::Departure { ship } => {
            state.ships.entry(ship).and_modify(|s| s.port = None);
        }
    };
    println!("new state: {:?}", state);
}

fn log_event(event: Event) {
    println!("Writing event: {:?}", event);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("log.jsonl")
        .unwrap();
    let as_json = serde_json::to_string(&event).expect("event should have been serialized as JSON");

    writeln!(file, "{}", as_json).expect("event should have been logged");
}

fn main() {
    let mut state = State {
        ships: HashMap::default(),
    };

    process_event(
        StateEvent::EnrolShip {
            ship: "hms_at_sea".into(),
        },
        &mut state,
    );
    process_event(
        StateEvent::EnrolShip {
            ship: "hms_hello".into(),
        },
        &mut state,
    );
    process_event(
        StateEvent::Arrival {
            ship: "hms_at_sea".into(),
            port: Port::SanFrancisco,
        },
        &mut state,
    );
    process_event(
        StateEvent::Arrival {
            ship: "hms_hello".into(),
            port: Port::Tokyo,
        },
        &mut state,
    );
    process_event(
        StateEvent::Departure {
            ship: "hms_hello".into(),
        },
        &mut state,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_process_events() {
        let mut state = State {
            ships: HashMap::default(),
        };
        process_event(
            StateEvent::EnrolShip {
                ship: "hms_at_sea".into(),
            },
            &mut state,
        );
        process_event(
            StateEvent::EnrolShip {
                ship: "hms_hello".into(),
            },
            &mut state,
        );
        process_event(
            StateEvent::Arrival {
                ship: "hms_at_sea".into(),
                port: Port::SanFrancisco,
            },
            &mut state,
        );
        process_event(
            StateEvent::Arrival {
                ship: "hms_hello".into(),
                port: Port::Tokyo,
            },
            &mut state,
        );
        process_event(
            StateEvent::Departure {
                ship: "hms_at_sea".into(),
            },
            &mut state,
        );

        assert_eq!(
            state.ships.len(),
            2,
            "there should be 2 ships but got {}",
            state.ships.len()
        );
        assert!(
            state.ships.get("hms_at_sea".into()).unwrap().port.is_none(),
            "hms_at_sea should be at sea",
        );

        assert_eq!(
            state.ships.get("hms_hello".into()).unwrap().port,
            Some(Port::Tokyo),
            "hms_hello should be docked at Tokyo",
        );
    }
}
