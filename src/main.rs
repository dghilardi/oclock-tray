use std::time::Duration;

use client::oclock::{client::OClockClient, dto::state::Task};
use futures::{executor::block_on, StreamExt};
use idle::{idle_evt_stream, IdleEvent};
use ksni;
use slint::{ComponentHandle, VecModel};
use tray::OClockTray;

mod client;
mod idle;
mod slintui;
mod tray;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init();

    let mut idle_evt_rx =
        idle_evt_stream(Duration::from_secs(120), None).expect("Error starting idle monitor");
    std::thread::spawn(|| {
        block_on(async move {
            let client = OClockClient::new();
            while let Some(evt) = idle_evt_rx.next().await {
                match evt {
                    IdleEvent::Idled => log::debug!("Idle detected"),
                    IdleEvent::Resumed { idle_start } => {
                        let dialog = crate::slintui::idletime::IdleTimeDialog::new()
                            .expect("Failed to create idle-time dialog");

                        let current_state =
                            client.get_state().expect("Error reading current state");

                        let enabled_tasks = current_state
                            .all_tasks
                            .iter()
                            .filter(|t| t.enabled > 0)
                            .map(|t| Task {
                                id: t.id,
                                enabled: t.enabled,
                                name: t.name.clone(),
                            })
                            .collect::<Vec<_>>();

                        let task_labels = enabled_tasks
                            .iter()
                            .map(|t| slint::SharedString::from(&t.name))
                            .collect::<Vec<_>>();

                        let weak_dialog = dialog.as_weak();
                        let client = client.clone();
                        dialog.set_available_tasks(VecModel::from_slice(&task_labels));
                        dialog.on_retro_switch_task(move |task_id, _timestamp, restore_prev| {
                            if task_id < 0 {
                                log::warn!("No task selected");
                            } else if let Some(task) = enabled_tasks.get(task_id as usize) {
                                match client.retro_switch_task(
                                    task.id as u64,
                                    idle_start,
                                    restore_prev,
                                ) {
                                    Ok(_) => log::info!("Task switched successfully"),
                                    Err(e) => log::error!("Failed to create task: {}", e),
                                };
                            } else {
                                log::warn!("Task id {task_id} out of bound");
                            }
                            weak_dialog
                                .unwrap()
                                .hide()
                                .expect("Failed to hide new task dialog");
                        });

                        dialog.run().expect("Error running idle-time dialog");
                        log::debug!("Resume detected");
                    }
                }
            }
            log::warn!("Idle thread terminated");
        })
    });

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
