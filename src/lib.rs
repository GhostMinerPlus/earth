use std::fs;
use toml::value::Array;

fn arg_v2toml(arg_v: &[String]) -> toml::Table {
    let mut config = toml::Table::default();
    let mut i = 0;
    while i < arg_v.len() {
        let word = &arg_v[i];
        i += 1;
        if word.starts_with("--") {
            let option = &word[2..];
            if arg_v[i].starts_with("--") {
                set_option(&mut config, option, "true");
            } else {
                set_option(&mut config, option, &arg_v[i]);
                i += 1;
            }
        }
    }
    config
}

fn parse_value(value: &str) -> toml::Value {
    if value == "false" {
        toml::Value::Boolean(false)
    } else if value == "true" {
        toml::Value::Boolean(true)
    } else if value.contains(',') {
        let value_v: Vec<&str> = value.split(',').filter(|s| !s.is_empty()).collect();
        let mut arr = Array::new();
        for value in value_v {
            arr.push(parse_value(value));
        }
        toml::Value::Array(arr)
    } else if value.contains('.') {
        match value.parse::<f64>() {
            Ok(f) => toml::Value::Float(f),
            Err(_) => toml::Value::String(value.to_string()),
        }
    } else if let Ok(i) = value.parse::<i64>() {
        toml::Value::Integer(i)
    } else {
        toml::Value::String(value.to_string())
    }
}

fn set_option(config: &mut toml::Table, option: &str, value: &str) {
    match option.find('.') {
        Some(pos) => {
            let sc = &mut config[&option[0..pos]];
            set_option(sc.as_table_mut().unwrap(), &option[pos + 1..], value);
        }
        None => {
            let v = parse_value(value);
            config.insert(option.to_string(), v);
        }
    };
}

fn merge_toml(left: &mut toml::Table, right: &toml::Table) {
    for (k, v) in left {
        if !right.contains_key(k) {
            continue;
        }
        if v.is_table() {
            merge_toml(v.as_table_mut().unwrap(), right[k].as_table().unwrap());
        } else {
            *v = right[k].clone();
        }
    }
}

fn merge_env(left: &mut toml::Table, root: &str) {
    for (k, v) in left {
        if v.is_table() {
            merge_env(v.as_table_mut().unwrap(), &format!("{root}{k}_"));
        } else if let Ok(new_value) = std::env::var(&format!("{root}{k}").to_uppercase()) {
            *v = parse_value(&new_value);
        }
    }
}

pub trait AsConfig: serde::ser::Serialize + serde::de::DeserializeOwned {
    fn merge_by_toml(&mut self, right: &toml::Table) {
        let mut temp: toml::Table = toml::from_str(&toml::to_string(self).unwrap()).unwrap();
        merge_toml(&mut temp, right);
        *self = toml::from_str(&toml::to_string(&temp).unwrap()).unwrap();
    }

    fn merge_by_env(&mut self, root: &str) {
        let mut temp: toml::Table = toml::from_str(&toml::to_string(self).unwrap()).unwrap();
        merge_env(&mut temp, root);
        *self = toml::from_str(&toml::to_string(&temp).unwrap()).unwrap();
    }

    fn merge_by_file(&mut self, pathname: &str) {
        match fs::read_to_string(pathname) {
            Ok(s) => self.merge_by_toml(&s.parse::<toml::Table>().unwrap()),
            Err(_) => (),
        }
    }

    fn merge_by_arg_v(&mut self, arg_v: &[String]) {
        self.merge_by_toml(&arg_v2toml(arg_v))
    }
}

pub use macros::*;

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
        app.merge_by_arg_v(&args);
        assert!(app.port == 8087);
    }
}
