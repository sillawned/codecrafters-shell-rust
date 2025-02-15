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
        let input = r"echo 'Hello World'";
        let tokens = tokenize(input);
        assert_eq!(tokens, vec![
            TokenType::Word("echo".to_string()),
            TokenType::SingleQuotedString("Hello World".to_string())
        ]);
    }

    #[test]
    fn test_double_quotes() {
        let input = r#"echo "Hello World""#;
        let tokens = tokenize(input);
        assert_eq!(tokens, vec![
            TokenType::Word("echo".to_string()),
            TokenType::DoubleQuotedString("Hello World".to_string())
        ]);
    }

    #[test]
    fn test_nested_quotes() {
        let input = r#"echo "hello 'world'""#;
        let tokens = tokenize(input);
        assert_eq!(tokens, vec![
            TokenType::Word("echo".to_string()),
            TokenType::DoubleQuotedString("hello 'world'".to_string())
        ]);
    }

    #[test]
    fn test_mixed_quotes() {
        let input = r#"echo 'hello "world"'"#;
        let tokens = tokenize(input);
        assert_eq!(tokens, vec![
            TokenType::Word("echo".to_string()),
            TokenType::SingleQuotedString("hello \"world\"".to_string())
        ]);
    }

    #[test]
    fn test_escaped_quotes() {
        let input = r#"echo hello\"world"#;
        let tokens = tokenize(input);
        assert_eq!(tokens, vec![
            TokenType::Word("echo".to_string()),
            TokenType::Word("hello\"world".to_string())
        ]);
    }

    // Additional tests to verify bash-like behavior
    #[test]
    fn test_bash_quote_behavior() {
        assert_eq!(
            tokenize(r#"echo "hello 'world'""#),
            tokenize(r#"echo 'hello "world"'"#)
        );
        
        // Test that quotes are preserved in output
        let input = r#"echo '"hello" world'"#;
        let tokens = tokenize(input);
        assert_eq!(tokens, vec![
            TokenType::Word("echo".to_string()),
            TokenType::SingleQuotedString("\"hello\" world".to_string())
        ]);
    }
}