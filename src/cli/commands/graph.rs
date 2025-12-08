use anyhow::Result;
use console::style;
use ptree::{TreeBuilder, print_tree};
use std::collections::HashMap;
use vesshelm::config::Config;

pub async fn run(config_path: &std::path::Path) -> Result<()> {
    println!(
        "{} Calculating dependency graph",
        style("==>").bold().green()
    );

    // Load configuration
    let config = Config::load_from_path(config_path)?;

    // Config::load_from_path already validates

    // We need to build the graph to know dependencies.
    // We can reuse logic from dag.rs, but we need the DAG structure itself to traverse.
    // dag::sort_charts returns sorted vector, consuming the graph logic.
    // Let's copy logic locally or expose graph from `dag.rs`?
    // Exposing graph would be cleaner but requires refactoring `dag.rs` to publicize `Dag`.
    // For now, let's just rebuild it locally or simpler:
    // Just build a parent->children map directly from config.
    // Actually, `dag.rs` uses `daggy`, let's just use the `depends` field directly.

    // Build Reverse Dependency Map: Parent -> [Children]
    // Where "Parent" is the independent chart (the Requirement).
    // Wait, let's define the tree structure again.
    // If A depends on B. B is dependency.
    // Deployment order: B then A.
    // Tree: B -> A.
    // "Reverse" dependency map is correct logic: Key=Dependency, Val=Dependent.

    let mut dependents_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut all_charts: HashMap<String, &vesshelm::config::Chart> = HashMap::new();
    let mut has_dependencies: HashMap<String, bool> = HashMap::new(); // Tracks if a chart HAS dependencies (is a child)

    for chart in &config.charts {
        all_charts.insert(chart.name.clone(), chart);

        // Initialize entry
        dependents_map.entry(chart.name.clone()).or_default();

        if let Some(deps) = &chart.depends {
            if !deps.is_empty() {
                has_dependencies.insert(chart.name.clone(), true);
                for dep in deps {
                    dependents_map
                        .entry(dep.clone())
                        .or_default()
                        .push(chart.name.clone());
                    // Also ensure dependency exists in has_dependencies map (default false)
                    has_dependencies.entry(dep.clone()).or_insert(false);
                }
            } else {
                has_dependencies.entry(chart.name.clone()).or_insert(false);
            }
        } else {
            has_dependencies.entry(chart.name.clone()).or_insert(false);
        }
    }

    // Identify Roots: Charts that do NOT depend on anything (they are purely dependencies or independent).
    // i.e., has_dependencies == false
    let mut roots: Vec<String> = has_dependencies
        .iter()
        .filter(|&(_, has_deps)| !has_deps)
        .map(|(name, _)| name.clone())
        .collect();

    roots.sort(); // Deterministic output

    // Build ptree
    // We can have multiple roots (independent trees).
    // ptree expects a single root usually. We can create a dummy root "Deployment Plan" or similar.

    let mut builder = TreeBuilder::new("Deployment Graph".to_string());

    for root in roots {
        add_node_recursive(&mut builder, &root, &dependents_map, 0);
    }

    let tree = builder.build();
    print_tree(&tree)?;

    Ok(())
}

fn add_node_recursive(
    builder: &mut TreeBuilder,
    current_node: &str,
    dependents_map: &HashMap<String, Vec<String>>,
    depth: usize,
) {
    let colors = [
        console::Color::Green,
        console::Color::Cyan,
        console::Color::Blue,
        console::Color::Magenta,
        console::Color::Yellow,
        console::Color::Red,
    ];

    let color = colors[depth % colors.len()];
    let styled_node = console::style(current_node).fg(color).to_string();

    builder.begin_child(styled_node);

    if let Some(dependents) = dependents_map.get(current_node) {
        let mut sorted_dependents = dependents.clone();
        sorted_dependents.sort();
        for dep in sorted_dependents {
            add_node_recursive(builder, &dep, dependents_map, depth + 1);
        }
    }

    builder.end_child();
}
