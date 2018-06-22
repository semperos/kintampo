pub fn example(x: Option<i32>) -> i32 {
    if let Some(x) = x {
        x
    } else {
        42
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        assert_eq!(42, example(None)); 
        assert_eq!(23, example(Some(23)));
    }
}
