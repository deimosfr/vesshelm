use console::style;

/// A structure to aggregate and print command execution summaries.
#[derive(Debug, Default)]
pub struct Summary {
    items: Vec<(String, String)>,
}

impl Summary {
    /// Create a new empty summary.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an item to the summary.
    /// The label will be padded for alignment during printing.
    /// The value should be pre-styled (e.g. using `console::style`).
    pub fn add<S: std::fmt::Display, V: std::fmt::Display>(
        &mut self,
        label: S,
        value: V,
    ) -> &mut Self {
        self.items.push((label.to_string(), value.to_string()));
        self
    }

    /// Print the summary to stdout.
    pub fn print(&self) {
        if self.items.is_empty() {
            return;
        }

        println!("\n{}", style("Summary:").bold());

        // Calculate max label length for padding
        let max_len = self
            .items
            .iter()
            .map(|(label, _)| label.len())
            .max()
            .unwrap_or(0);

        for (label, value) in &self.items {
            println!("  {:<width$} {}", label, value, width = max_len + 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use console::style;

    #[test]
    fn test_summary_formatting() {
        let mut summary = Summary::new();
        summary.add("Short", "Value");
        summary.add("Longer Label", style("Value2").red().to_string());

        // We can't easily capture stdout here, but we verify it compiles and runs.
        // For strict verification we would need a writer injector, but println is standard for this CLI tool.
        summary.print();
    }
}
