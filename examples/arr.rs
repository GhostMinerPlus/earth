use earth::AsConfig;

#[derive(serde::Serialize, serde::Deserialize, AsConfig, Debug)]
struct Config {
    name: String,
    port_v: Vec<u16>,
}

fn main() {
    let mut arg_v: Vec<String> = std::env::args().collect();
    arg_v.remove(0);
    let mut config = Config {
        name: "".to_string(),
        port_v: vec![8080],
    };
    println!("{:?}", arg_v);
    config.merge_by_args(&arg_v);
    println!("{:?}", config);
}
