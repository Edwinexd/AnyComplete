#[cfg(test)]
pub mod test_remove_overlapping {

    use crate::remove_overlapping;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
    #[test]
    fn test_remove_overlapping() {
        assert_eq!(remove_overlapping("Yellow! ", "hello world hello"), "hello world hello");
    }

    #[test]
    fn no_overlapping() {
        assert_eq!(remove_overlapping("Hello! ", "hello world"), "hello world");
    }

    #[test]
    fn no_overlapping_contains_input() {
        assert_eq!(remove_overlapping("Hello! ", "hello world Hello! "), "hello world Hello! ");
    }

    #[test]
    fn overlapping() {
        assert_eq!(remove_overlapping("In conclusion: ", "In conclusion: hello world"), "hello world");
    }

    #[test]
    fn overlapping_contains_input() {
        assert_eq!(remove_overlapping("In conclusion: ", "In conclusion: hello world In conclusion: "), "hello world In conclusion: ");
    }

    #[test]
    fn partial_overlapping() {
        assert_eq!(remove_overlapping("So in conclusion: ", "in conclusion: hello world"), "hello world");
    }

    #[test]
    fn partial_overlapping_contains_input() {
        assert_eq!(remove_overlapping("So in conclusion: ", "in conclusion: hello world So in conclusion: "), "hello world So in conclusion: ");
    }

}