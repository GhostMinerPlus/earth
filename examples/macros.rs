use earth::AsConfig;

#[derive(serde::Serialize, serde::Deserialize, AsConfig, Debug)]
struct Config {
    name: String,
    port: u16,
}

fn main() {
    let mut arg_v: Vec<String> = std::env::args().collect();
    arg_v.remove(0);
    let mut config = Config {
        name: "".to_string(),
        port: 8080,
    };
    config.merge_by_file("earth.toml");
    config.merge_by_arg_v(&arg_v);
    println!("{:?}", config);
}
