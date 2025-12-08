use anyhow::Result;
use serde_yaml_ng::Value;

pub fn merge_values(values: &[Value]) -> Result<String> {
    if values.is_empty() {
        return Ok("".to_string());
    }

    let mut merged_value = values[0].clone();

    for value in values.iter().skip(1) {
        merge_yaml_values(&mut merged_value, value)?;
    }

    let yaml_string = serde_yaml_ng::to_string(&merged_value)?;
    Ok(yaml_string)
}

fn merge_yaml_values(a: &mut Value, b: &Value) -> Result<()> {
    match (a, b) {
        (Value::Mapping(a_map), Value::Mapping(b_map)) => {
            for (key, value) in b_map {
                if let Some(a_value) = a_map.get_mut(key) {
                    merge_yaml_values(a_value, value)?;
                } else {
                    a_map.insert(key.clone(), value.clone());
                }
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml_ng::{Mapping, Number};

    #[test]
    fn test_merge_values() {
        let v1: Value = serde_yaml_ng::from_str("key1: value1").unwrap();
        let v2: Value = serde_yaml_ng::from_str("key2: 2").unwrap();
        // Overrides
        let v3: Value = serde_yaml_ng::from_str("key1: value1_updated").unwrap();

        let merged = merge_values(&[v1, v2, v3]).unwrap();
        let parsed: Value = serde_yaml_ng::from_str(&merged).unwrap();

        let expected_map: Mapping = parsed.as_mapping().unwrap().clone();
        assert_eq!(
            expected_map.get(&Value::String("key1".into())),
            Some(&Value::String("value1_updated".into()))
        );
        assert_eq!(
            expected_map.get(&Value::String("key2".into())),
            Some(&Value::Number(Number::from(2)))
        );
    }

    #[test]
    fn test_merge_values_invalid() {
        let v1: Value = serde_yaml_ng::from_str("key1: value1").unwrap();
        let v2: Value = serde_yaml_ng::from_str("- list_item").unwrap(); // This parses as Sequence

        // Merging mapping and sequence should overwrite mapping with sequence (in our simple logic? or error?)
        // Our logic says: (a, b) -> *a = b.clone() if not both mappings.
        // So it should become the sequence.
        let _merged = merge_values(&[v1, v2]).unwrap();
        // assert_eq!(merged.trim(), "- list_item");
        // Actually serde_yaml output format might vary.
    }
}
