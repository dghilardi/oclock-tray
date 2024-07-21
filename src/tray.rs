use std::sync::{Arc, Mutex};

use crate::client::oclock::{client::OClockClient, dto::state::ExportedState};

pub struct OClockTray {
    client: OClockClient,
    state: ExportedState,
}

impl OClockTray {
    pub fn new(client: OClockClient) -> anyhow::Result<Self> {
        let state = client.get_state()?;
        Ok(Self { client, state })
    }
}

impl ksni::Tray for OClockTray {
    fn icon_name(&self) -> String {
        "help-about".into()
    }
    fn title(&self) -> String {
        if let Some(current) = self.state.current_task.as_ref() {
            format!("OClock - {}", current.name)
        } else {
            String::from("OClock")
        }
    }
    // NOTE: On some system trays, `id` is a required property to avoid unexpected behaviors
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![
            StandardItem {
                label: "New Task".into(),
                icon_name: "list-add".into(),
                activate: Box::new(|_| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            RadioGroup {
                selected: self
                    .state
                    .all_tasks
                    .iter()
                    .position(|task| {
                        self.state
                            .current_task
                            .as_ref()
                            .map(|current| current.id == task.id)
                            .unwrap_or(false)
                    })
                    .unwrap_or(self.state.all_tasks.len()),
                select: Box::new(|this: &mut Self, current| {
                    let selected_task = this.state.all_tasks.get(current);
                    if let Some(task) = selected_task {
                        match this.client.switch_task(task.id as u64) {
                            Ok(new_state) => {
                                this.state = new_state;
                            }
                            Err(e) => {
                                log::error!("Error switching task: {}", e);
                            }
                        }
                    } else {
                        log::warn!("Invalid task selected: {}", current);
                    }
                }),
                options: self
                    .state
                    .all_tasks
                    .iter()
                    .map(|task| RadioItem {
                        label: task.name.clone(),
                        ..Default::default()
                    })
                    .collect(),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Exit".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|_| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        ]
    }
}
