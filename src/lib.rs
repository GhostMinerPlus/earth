use std::fs;
use toml::value::Array;

pub use macros::*;

pub trait AsConfig: serde::ser::Serialize + serde::de::DeserializeOwned {
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

    fn merge_by_args(&mut self, args: &[String]) {
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

fn properties2toml(arg_v: &[String]) -> toml::Table {
    let mut table = toml::Table::default();
    let mut i = 0;
    while i < arg_v.len() {
        let word = &arg_v[i];
        i += 1;
        if word.starts_with("--") {
            let option = &word[2..];
            toml_insert_option(&mut table, option, &arg_v[i]);
            i += 1;
        }
    }
    table
}

fn toml_insert_option(lc: &mut toml::Table, option: &str, value: &str) {
    match option.find('.') {
        Some(s) => {
            match lc.get_mut(&option[0..s].to_string()) {
                Some(sc) => {
                    toml_insert_option(&mut sc.as_table_mut().unwrap(), &option[s + 1..], value);
                }
                None => {
                    let mut sc = toml::Table::default();
                    toml_insert_option(&mut sc, &option[s + 1..], value);
                    lc.insert(option[0..s].to_string(), toml::Value::Table(sc));
                }
            };
        }
        None => {
            if value.contains(',') {
                let value_v: Vec<&str> = value.split(',').filter(|s| !s.is_empty()).collect();
                if let None = lc.get(option) {
                    lc.insert(option.to_string(), toml::Value::Array(Array::new()));
                }
                let arr = lc[option].as_array_mut().unwrap();
                for value in value_v {
                    if value.is_empty() {
                        arr.push(toml::Value::Boolean(true));
                    } else {
                        arr.push(match value.find('.') {
                            Some(_) => match value.parse::<f64>() {
                                Ok(o) => toml::Value::Float(o),
                                Err(_) => toml::Value::String(value.to_string()),
                            },
                            None => match value.parse::<i64>() {
                                Ok(o) => toml::Value::Integer(o),
                                Err(_) => toml::Value::String(value.to_string()),
                            },
                        });
                    }
                }
            } else {
                if value.is_empty() {
                    lc.insert(option.to_string(), toml::Value::Boolean(true));
                } else {
                    lc.insert(
                        option.to_string(),
                        match value.find('.') {
                            Some(_) => match value.parse::<f64>() {
                                Ok(o) => toml::Value::Float(o),
                                Err(_) => toml::Value::String(value.to_string()),
                            },
                            None => match value.parse::<i64>() {
                                Ok(o) => toml::Value::Integer(o),
                                Err(_) => toml::Value::String(value.to_string()),
                            },
                        },
                    );
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::AsConfig;

    #[derive(serde::Deserialize, serde::Serialize)]
    struct App {
        name: String,
        port: u16,
    }

    impl AsConfig for App {}

    #[test]
    fn test_config() {
        let args = std::vec!["--port".to_string(), "8087".to_string()];
        let mut app = App {
            name: "".to_string(),
            port: 8080,
        };
        app.merge_by_file("config.toml");
        app.merge_by_args(&args);
        assert!(app.port == 8087);
    }
}
