#[test]
fn test_parameter_expansion_space_before_negative() {
    let mut shell = Shell::new();
    shell.variables.insert("TEXT".to_string(), "hello_world".to_string());
    
    let result = shell.expand_variables("${TEXT: -5}");
    assert_eq!(result, "world", "Should expand to 'world', got: '{}'", result);
}
