slint::include_modules!();

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

    pub fn update_state(&mut self, state: ExportedState) {
        self.state = state;
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
        let client = self.client.clone();
        vec![
            StandardItem {
                label: "New Task".into(),
                icon_name: "list-add".into(),
                activate: Box::new(move |t: &mut OClockTray| {
                    let dialog = NewTaskDialog::new().expect("Failed to create new task dialog");
                    let weak_dialog = dialog.as_weak();
                    let client = client.clone();
                    dialog.on_create_new_task(move |task_name| {
                        match client.new_task(task_name.as_str().to_string()) {
                            Ok(_) => log::info!("Task created successfully"),
                            Err(e) => log::error!("Failed to create task: {}", e),
                        };
                        weak_dialog
                            .unwrap()
                            .hide()
                            .expect("Failed to hide new task dialog");
                    });
                    dialog.run().expect("Failed to run new task dialog");
                }),
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
