use std::fs;

pub trait Config: serde::ser::Serialize + serde::de::DeserializeOwned {
    fn merge_by_toml(&mut self, toml: &toml::Table) {
        let mut temp: toml::Table = toml::from_str(&toml::to_string(self).unwrap()).unwrap();
        right_merge_config(&mut temp, toml);
        *self = toml::from_str(&toml::to_string(&temp).unwrap()).unwrap();
    }

    fn merge_by_file(&mut self, pathname: &str) {
        match fs::read_to_string(pathname) {
            Ok(o) => self.merge_by_toml(&o.parse::<toml::Table>().unwrap()),
            Err(_) => (),
        }
    }

    fn merge_by_args(&mut self, args: &std::vec::Vec<String>) {
        self.merge_by_toml(&properties2toml(args))
    }
}

fn right_merge_config(lc: &mut toml::Table, rc: &toml::Table) {
    for (k, v) in rc {
        match lc.get_mut(k) {
            Some(o) => {
                if o.is_table() {
                    right_merge_config(o.as_table_mut().unwrap(), v.as_table().unwrap());
                } else {
                    lc.insert(k.clone(), v.clone());
                }
            }
            None => {
                lc.insert(k.clone(), v.clone());
            }
        };
    }
}

fn properties2toml(args: &std::vec::Vec<String>) -> toml::Table {
    let mut table = toml::Table::default();
    let mut i = 0;
    loop {
        let word = &args[i];
        i += 1;
        if word.starts_with("--") {
            let option = &word[2..];
            let start = i;
            while i < args.len() && !args[i].starts_with("--") {
                i += 1;
            }
            toml_insert_option(&mut table, option, &args[start..i]);
        }
        if i >= args.len() {
            break;
        }
    }
    table
}

fn toml_insert_option(lc: &mut toml::Table, option: &str, args: &[String]) {
    match option.find('.') {
        Some(s) => {
            match lc.get_mut(&option[0..s].to_string()) {
                Some(sc) => {
                    toml_insert_option(&mut sc.as_table_mut().unwrap(), &option[s + 1..], args);
                }
                None => {
                    let mut sc = toml::Table::default();
                    toml_insert_option(&mut sc, &option[s + 1..], args);
                    lc.insert(option[0..s].to_string(), toml::Value::Table(sc));
                }
            };
        }
        None => {
            if args.is_empty() {
                lc.insert(option.to_string(), toml::Value::Boolean(true));
            } else {
                lc.insert(
                    option.to_string(),
                    match args[0].find('.') {
                        Some(_) => match args[0].parse::<f64>() {
                            Ok(o) => toml::Value::Float(o),
                            Err(_) => toml::Value::String(args[0].to_string()),
                        },
                        None => match args[0].parse::<i64>() {
                            Ok(o) => toml::Value::Integer(o),
                            Err(_) => toml::Value::String(args[0].to_string()),
                        },
                    },
                );
            }
        }
    };
}

pub use macros::*;

#[cfg(test)]
mod tests {
    use crate::Config;

    #[derive(serde::Deserialize, serde::Serialize)]
    struct App {
        name: String,
        port: u16,
    }

    impl Config for App {}

    #[test]
    fn config() {
        let args = std::vec!["--port".to_string(), "8087".to_string()];
        let mut app = App {
            name: "".to_string(),
            port: 8080,
        };
        app.merge_by_file("earth.toml");
        app.merge_by_args(&args);
        assert!(app.port == 8087);
    }
}
