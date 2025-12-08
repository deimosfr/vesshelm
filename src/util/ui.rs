use console::style;

/// Formats a step message with the standard arrow prefix.
/// Example output: "==> Message" (where ==> is bold green/blue depending on context)
/// But sticking to the requested pattern: style("==>").bold().green()
pub fn style_step(msg: &str) -> String {
    format!("{} {}", style("==>").bold().green().bright(), msg)
}

/// Same as style_step but prints to stdout directly.
pub fn print_step(msg: &str) {
    println!("{} {}", style("=>").bold().green().bright(), msg)
}
