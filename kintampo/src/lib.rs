extern crate edn;
#[macro_use]
extern crate log;

pub fn parse_edn_vector(s: &str) -> Vec<String> {
    match edn::parser::Parser::new(s).read().unwrap().expect("bad EDN") {
        edn::Value::Vector(items) => {
            items.into_iter().map(|i| match i { edn::Value::String(s) => s, _ => "".to_owned()}).collect()
        },
        _ => {
            error!("Only EDN vectors of strings supported.");
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_edn_vector() {
        assert_eq!(vec!["a".to_owned(), "b".to_owned()], parse_edn_vector("[\"a\",\"b\"]"));
    }

    #[test]
    fn test_parse_edn_vector_empty_as_error() {
        let expected: Vec<String> = vec![];
        assert_eq!(expected, parse_edn_vector(":something-else"));
    }
}
