fn slint_build_ui(path: &str, env: &str) {
    if slint_build::compile(path).is_ok() {
        println!(
            "cargo:rustc-env=SLINT_INCLUDE_{env}={}/{}.rs",
            std::env::var("OUT_DIR").unwrap(),
            path.split('/')
                .last()
                .and_then(|p| p.strip_suffix(".slint"))
                .unwrap_or_default()
        );
        println!("cargo:info=compile {path} success\n");
    }
}

fn main() {
    slint_build_ui("ui/new-task.slint", "NEW_TASK");
    slint_build_ui("ui/idle-time-dialog.slint", "IDLE_TIME");
}
