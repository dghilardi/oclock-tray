use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ExportedState {
    pub current_task: Option<Task>,
    pub all_tasks: Vec<Task>,
}

#[derive(Deserialize, Debug)]
pub struct Task {
    pub id: i32,
    pub enabled: i32,
    pub name: String,
}
