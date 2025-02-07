use crate::tokenizer;

#[cfg(test)]
mod tests {

    #[test]
    fn test_escape_character() {
        let input = r"echo Hello\ World";
        let tokens = tokenize(input);
        assert_eq!(tokens, vec![
            TokenType::Word("echo".to_string()),
            TokenType::Word("Hello World".to_string())
        ]);
    }

    #[test]
    fn test_single_quotes() {
        let input = "echo 'Hello World'";
        let tokens = tokenize(input);
        assert_eq!(tokens, vec![
            TokenType::Word("echo".to_string()),
            TokenType::QuotedString("Hello World".to_string())
        ]);
    }

    #[test]
    fn test_double_quotes() {
        let input = "echo \"Hello World\"";
        let tokens = tokenize(input);
        assert_eq!(tokens, vec![
            TokenType::Word("echo".to_string()),
            TokenType::QuotedString("Hello World".to_string())
        ]);
    }

    #[test]
    fn test_double_quotes_with_variable() {
        let input = "echo \"Home directory is $HOME\"";
        let tokens = tokenize(input);
        assert_eq!(tokens, vec![
            TokenType::Word("echo".to_string()),
            TokenType::QuotedString("Home directory is $HOME".to_string())
        ]);
    }

    #[test]
    fn test_ansi_c_quoting() {
        let input = "echo $'Hello\\nWorld'";
        let tokens = tokenize(input);
        assert_eq!(tokens, vec![
            TokenType::Word("echo".to_string()),
            TokenType::QuotedString("Hello\nWorld".to_string())
        ]);
    }

    #[test]
    fn test_locale_specific_translation() {
        let input = "echo $\"Hello World\"";
        let tokens = tokenize(input);
        assert_eq!(tokens, vec![
            TokenType::Word("echo".to_string()),
            TokenType::QuotedString("Hello World".to_string())
        ]);
    }
}