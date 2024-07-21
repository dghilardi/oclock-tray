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
}
