use std::{
    collections::HashMap,
    ffi::CString,
    time::{Duration, SystemTime},
};

use anyhow::Context;
use futures::channel::mpsc::{channel, Receiver, Sender};
use wayrs_client::{
    global::GlobalsExt, object::Proxy, protocol::WlSeat, Connection, EventCtx, IoMode,
};
use wayrs_protocols::ext_idle_notify_v1::{
    ext_idle_notification_v1, ExtIdleNotificationV1, ExtIdleNotifierV1,
};
use wayrs_utils::seats::{SeatHandler, Seats};

pub fn idle_evt_stream(
    timeout: Duration,
    seat_name: Option<CString>,
) -> anyhow::Result<Receiver<IdleEvent>> {
    let (mut conn, globals) = Connection::connect_and_collect_globals()?;

    let (tx, rx) = channel(100);
    let mut state = State {
        exit: false,
        seats: Seats::bind(&mut conn, &globals),
        seat_names: HashMap::default(),

        idle_event_tx: tx,
        idle_duration: timeout,
        idle_start: None,
    };
    conn.blocking_roundtrip()?; // Receive seat names.
    conn.dispatch_events(&mut state);

    let seat = match seat_name {
        Some(name) => state
            .seat_names
            .get(&name)
            .copied()
            .with_context(|| format!("seat '{}' not found", name.to_string_lossy()))?,
        None => state.seats.iter().next().context("no seats found")?,
    };
    let idle_notifier = globals.bind::<ExtIdleNotifierV1, _>(&mut conn, 1..=1)?;

    idle_notifier.get_idle_notification_with_cb(
        &mut conn,
        timeout.as_millis() as u32,
        seat,
        callback,
    );

    std::thread::spawn(move || {
        while !state.exit {
            conn.flush(IoMode::Blocking)?;
            conn.recv_events(IoMode::Blocking)?;
            conn.dispatch_events(&mut state);
        }
        Ok::<_, anyhow::Error>(())
    });

    Ok(rx)
}

fn callback<'a>(arg: EventCtx<'a, State, ExtIdleNotificationV1>) {
    match arg.event {
        ext_idle_notification_v1::Event::Idled => {
            arg.state.idle_start = Some(SystemTime::now() - arg.state.idle_duration);

            let out = arg.state.idle_event_tx.try_send(IdleEvent::Idled);
            if let Err(err) = out {
                log::error!("Error sending idle event - {err}");
            }
        }
        ext_idle_notification_v1::Event::Resumed => {
            if let Some(idle_start) = arg.state.idle_start.take() {
                let out = arg
                    .state
                    .idle_event_tx
                    .try_send(IdleEvent::Resumed { idle_start });
                if let Err(err) = out {
                    log::error!("Error sending resumed event - {err}");
                }
            }
        }
        _ => {}
    };
}

pub enum IdleEvent {
    Idled,
    Resumed { idle_start: SystemTime },
}

struct State {
    exit: bool,
    seats: Seats,
    seat_names: HashMap<CString, WlSeat>,

    idle_event_tx: futures::channel::mpsc::Sender<IdleEvent>,
    idle_duration: Duration,
    idle_start: Option<SystemTime>,
}

impl SeatHandler for State {
    fn get_seats(&mut self) -> &mut Seats {
        &mut self.seats
    }

    fn seat_name(&mut self, _: &mut Connection<Self>, wl_seat: WlSeat, name: CString) {
        self.seat_names.insert(name, wl_seat);
    }
}
