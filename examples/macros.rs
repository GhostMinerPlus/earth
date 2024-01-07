use earth::AsConfig;

#[derive(serde::Serialize, serde::Deserialize, AsConfig)]
struct App {
    name: String,
    port: u16,
}

fn main() {
    let args = std::vec!["--port".to_string(), "8087".to_string()];
    let mut app = App {
        name: "".to_string(),
        port: 8080,
    };
    app.merge_by_file("earth.toml");
    app.merge_by_args(&args);
}
