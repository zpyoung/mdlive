use minijinja::value::Value;
use std::collections::{BTreeMap, HashMap};

pub(crate) fn build_file_tree(paths: &[String]) -> Vec<Value> {
    build_tree_level(paths, "")
}

fn build_tree_level(paths: &[String], prefix: &str) -> Vec<Value> {
    let mut dirs: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut files: Vec<String> = Vec::new();

    for path in paths {
        if let Some(slash_pos) = path.find('/') {
            let dir_name = &path[..slash_pos];
            let rest = &path[slash_pos + 1..];
            dirs.entry(dir_name.to_string())
                .or_default()
                .push(rest.to_string());
        } else {
            files.push(path.clone());
        }
    }

    let mut items: Vec<(String, Value)> = Vec::new();

    for (dir_name, sub_paths) in &dirs {
        let dir_prefix = if prefix.is_empty() {
            dir_name.clone()
        } else {
            format!("{}/{}", prefix, dir_name)
        };
        let children = build_tree_level(sub_paths, &dir_prefix);
        let mut map = HashMap::new();
        map.insert("name".to_string(), Value::from(dir_name.clone()));
        map.insert("is_dir".to_string(), Value::from(true));
        map.insert("dir_path".to_string(), Value::from(dir_prefix.clone()));
        map.insert("children".to_string(), Value::from(children));
        items.push((dir_name.to_lowercase(), Value::from_object(map)));
    }

    for file_name in &files {
        let full_path = if prefix.is_empty() {
            file_name.clone()
        } else {
            format!("{}/{}", prefix, file_name)
        };
        let mut map = HashMap::new();
        map.insert("name".to_string(), Value::from(file_name.clone()));
        map.insert("path".to_string(), Value::from(full_path));
        map.insert("is_dir".to_string(), Value::from(false));
        items.push((file_name.to_lowercase(), Value::from_object(map)));
    }

    items.sort_by(|a, b| a.0.cmp(&b.0));
    items.into_iter().map(|(_, v)| v).collect()
}
