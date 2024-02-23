use std::collections::BTreeMap;

pub fn json_obj_to_btreemap(json_object: serde_json::Value) -> BTreeMap<String, String> {
    let mut labels_tree: BTreeMap<String, String> = BTreeMap::new();

    for (key, value) in json_object.as_object().unwrap() {
        labels_tree.insert(key.to_owned(), value.as_str().unwrap().to_string());
    }

    labels_tree
}
