use rabe_bn::*;
use std::collections::HashSet;


pub fn usize_to_fr(_i: usize) -> Fr {
    return Fr::from_str(&_i.to_string()).unwrap();
}

pub fn contains(data: &Vec<String>, value: &String) -> bool {
    let len = data.into_iter()
        .filter(|&i| i == value)
        .collect::<Vec<_>>()
        .len();
    return len >= 1;
}

// used to check if a set of attributes is a subset of another
pub fn is_subset(_subset: &Vec<String>, _attr: &Vec<String>) -> bool {
    let super_set: HashSet<_> = _attr.iter().cloned().collect();
    let sub_set: HashSet<_> = _subset.iter().cloned().collect();
    return sub_set.is_subset(&super_set);
}

#[cfg(test)]
mod tests {

    use super::*;
    use utils::policy::pest::{PolicyLanguage, parse};

    #[test]
    fn test_traverse() {
        let policy_false = String::from(r#"joking-around?"#);
        let policy1 = String::from(r#"{"name": "and", "children": [{"name": "A"}, {"name": "B"}]}"#);
        let policy2 = String::from(r#"{"name": "or", "children": [{"name": "A"}, {"name": "B"}]}"#);
        let policy3 = String::from(
            r#"{"name": "and", "children": [{"or": [{"name": "C"}, {"name": "D"}]}, {"name": "B"}]}"#,
        );
        let mut _set0: Vec<String> = Vec::new();
        _set0.push(String::from("X"));
        _set0.push(String::from("Y"));

        let mut _set1: Vec<String> = Vec::new();
        _set1.push(String::from("A"));
        _set1.push(String::from("B"));

        let mut _set2: Vec<String> = Vec::new();
        _set2.push(String::from("C"));
        _set2.push(String::from("D"));

        let mut _set3: Vec<String> = Vec::new();
        _set3.push(String::from("A"));
        _set3.push(String::from("B"));
        _set3.push(String::from("C"));
        _set3.push(String::from("D"));

        assert_eq!(parse(policy_false.as_ref(), PolicyLanguage::JsonPolicy).is_ok() , false);

        match parse(policy1.as_ref(), PolicyLanguage::JsonPolicy) {
            Ok(pol) => {
                assert_eq!(traverse_policy(&_set0, &pol, PolicyType::Leaf), false);
                assert_eq!(traverse_policy(&_set1, &pol, PolicyType::Leaf), true);
                assert_eq!(traverse_policy(&_set2, &pol, PolicyType::Leaf), false);
                assert_eq!(traverse_policy(&_set3, &pol, PolicyType::Leaf), true);
            },
            Err(e) => println!("test_traverse: could not parse policy1 {}", e)
        }

        match parse(policy2.as_ref(), PolicyLanguage::JsonPolicy) {
            Ok(pol) => {
                assert_eq!(traverse_policy(&_set1, &pol, PolicyType::Leaf), true);
                assert_eq!(traverse_policy(&_set2, &pol, PolicyType::Leaf), false);
                assert_eq!(traverse_policy(&_set3, &pol, PolicyType::Leaf), true);
            },
            Err(e) => println!("test_traverse: could not parse policy2 {}", e)
        }

        match parse(policy3.as_ref(), PolicyLanguage::JsonPolicy) {
            Ok(pol) => {
                assert_eq!(traverse_policy(&_set1, &pol, PolicyType::Leaf), false);
                assert_eq!(traverse_policy(&_set2, &pol, PolicyType::Leaf), false);
                assert_eq!(traverse_policy(&_set3, &pol, PolicyType::Leaf), true);
            },
            Err(e) => println!("test_traverse: could not parse policy3 {}", e)
        }
    }
}
