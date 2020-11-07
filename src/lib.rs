pub fn parse_params(args: &[String]) -> (&str, &str) {
    let args = args;
    let query: &str = &args[1].as_str();
    let filename: &str = &args[2].as_str();
    return (query, filename);
}
