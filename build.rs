fn main() {
    let config = slint_build::CompilerConfiguration::new().with_style("cosmic".into());
    slint_build::compile_with_config("ui/board_ui.slint", config).unwrap();
}
