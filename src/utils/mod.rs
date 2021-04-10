pub mod bookmarks;
pub mod google;

pub fn get_command_from_query_string(query: &str) -> (&str, &str) {
    if query.contains(' ') {
        let index_of_space = query.find(' ').unwrap_or(0);
        return (&query[..index_of_space], &query[index_of_space + 1..]);
    }

    (query, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_command_from_query_string_no_whitespace() {
        let actual = get_command_from_query_string("tw");
        let expected = "tw";
        assert_eq!(expected, actual);
    }

    #[test]
    fn get_command_from_query_string_with_whitespace() {
        let actual = get_command_from_query_string("tw @fbOpenSource");
        let expected = "tw";
        assert_eq!(expected, actual);
    }
}
