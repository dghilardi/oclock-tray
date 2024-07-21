use client::oclock::client::OClockClient;
use ksni;
use tray::OClockTray;

mod client;
pub mod tray;

fn main() {
    env_logger::init();

    let client = OClockClient::new();

    let service =
        ksni::TrayService::new(OClockTray::new(client).expect("Failed to create tray service"));

    let handle = service.handle();
    service.spawn();

    std::thread::sleep(std::time::Duration::from_secs(5));
    // We can modify the tray
    //handle.update(|tray: &mut MyTray| {
    //    tray.checked = true;
    //});
    // Run forever
    loop {
        std::thread::park();
    }
}
