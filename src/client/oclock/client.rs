use std::time::{SystemTime, UNIX_EPOCH};

use futures::SinkExt;
use nng::{
    options::{protocol::pubsub::Subscribe, Options},
    Socket,
};
use oclock::core::constants::SERVER_SUB_URL;

use super::dto::state::ExportedState;

#[derive(Clone)]
pub struct OClockClient {}

impl OClockClient {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_state(&self) -> anyhow::Result<ExportedState> {
        let result = oclock::client::handler::invoke_server::<
            oclock::dto::command::OClockClientCommand,
            ExportedState,
        >(oclock::dto::command::OClockClientCommand::JsonState)?;
        Ok(result)
    }

    pub fn new_task(&self, name: String) -> anyhow::Result<ExportedState> {
        let result = oclock::client::handler::invoke_server::<
            oclock::dto::command::OClockClientCommand,
            ExportedState,
        >(oclock::dto::command::OClockClientCommand::JsonPushTask { name })?;
        Ok(result)
    }

    pub fn switch_task(&self, task_id: u64) -> anyhow::Result<ExportedState> {
        let result =
            oclock::client::handler::invoke_server::<
                oclock::dto::command::OClockClientCommand,
                ExportedState,
            >(oclock::dto::command::OClockClientCommand::JsonSwitchTask { task_id })?;
        Ok(result)
    }

    pub fn retro_switch_task(
        &self,
        task_id: u64,
        timestamp: SystemTime,
        keep_previous_task: bool,
    ) -> anyhow::Result<ExportedState> {
        let timestamp = timestamp
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let result = oclock::client::handler::invoke_server::<
            oclock::dto::command::OClockClientCommand,
            ExportedState,
        >(
            oclock::dto::command::OClockClientCommand::JsonRetroSwitchTask {
                task_id,
                timestamp,
                keep_previous_task,
            },
        )?;
        Ok(result)
    }

    pub fn spawn_listener(
        &self,
    ) -> anyhow::Result<futures::channel::mpsc::Receiver<ExportedState>> {
        let s = Socket::new(nng::Protocol::Sub0)?;
        s.dial(SERVER_SUB_URL)?;
        let all_topics = vec![];
        s.set_opt::<Subscribe>(all_topics)?;

        let (mut tx, rx) = futures::channel::mpsc::channel(100);
        std::thread::spawn(move || {
            while let Ok(msg) = s.recv() {
                match serde_json::from_slice::<ExportedState>(&msg[..]) {
                    Ok(state) => {
                        let out = tx.try_send(state);
                        if let Err(err) = out {
                            log::error!("Error sending state - {err}");
                        }
                    }
                    Err(err) => log::error!("Error deserializing state - {err}"),
                }
            }
        });
        Ok(rx)
    }
}
