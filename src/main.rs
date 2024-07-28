use client::oclock::client::OClockClient;
use futures::StreamExt;
use ksni;
use tray::OClockTray;

mod client;
pub mod tray;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init();

    let client = OClockClient::new();

    let mut rx = client.spawn_listener().expect("Error spaning listener");

    let service =
        ksni::TrayService::new(OClockTray::new(client).expect("Failed to create tray service"));

    let handle = service.handle();
    service.spawn();

    while let Some(state) = rx.next().await {
        handle.update(|tray: &mut OClockTray| {
            tray.update_state(state);
        })
    }

    loop {
        std::thread::park();
    }
}
