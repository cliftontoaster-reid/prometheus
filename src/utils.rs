pub fn concatenate_with_and(elements: Vec<String>) -> String {
  match elements.len() {
    0 => String::new(),
    1 => elements[0].to_string(),
    _ => {
      let mut result = elements[..elements.len() - 1].join(", ");
      result.push_str(" and ");
      result.push_str(elements.last().unwrap());
      result
    }
  }
}

pub fn add_if_not_present(arr: &mut Vec<String>, element: &str) {
  if !arr.contains(&element.to_string()) {
    arr.push(element.to_string());
  }
}

pub fn lowercase_and_replace(s: &str) -> String {
  let mut result = s.to_lowercase();
  result = result.replace("_", " ");
  result
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_add_if_not_present() {
    let mut fruits = vec!["apple".to_string(), "banana".to_string()];
    add_if_not_present(&mut fruits, "apple");
    add_if_not_present(&mut fruits, "cherry");

    assert_eq!(
      fruits,
      vec![
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string()
      ]
    );
  }

  #[test]
  fn test_add_if_not_present_duplicate() {
    let mut fruits = vec!["apple".to_string(), "banana".to_string()];
    add_if_not_present(&mut fruits, "apple");

    assert_eq!(fruits, vec!["apple".to_string(), "banana".to_string()]);
  }

  #[test]
  fn test_lowercase_and_replace() {
    assert_eq!(
      lowercase_and_replace("Hello_World_and_Some_Other_THINGS"),
      "hello world and some other things"
    );
    assert_eq!(
      lowercase_and_replace("Multiple_Underscores___Here"),
      "multiple underscores   here"
    );
    assert_eq!(lowercase_and_replace("ALL_UPPERCASE"), "all uppercase");
    assert_eq!(lowercase_and_replace("lowercase"), "lowercase");
    assert_eq!(
      lowercase_and_replace("   leading_spaces"),
      "   leading spaces"
    );
    assert_eq!(
      lowercase_and_replace("trailing_spaces   "),
      "trailing spaces   "
    );
    assert_eq!(
      lowercase_and_replace("   leading_and_trailing_spaces   "),
      "   leading and trailing spaces   "
    );
    assert_eq!(lowercase_and_replace(""), "");
  }
}