use crate::config::Chart;
use anyhow::{Result, anyhow};
use daggy::{Dag, NodeIndex, Walker, WouldCycle};
use std::collections::HashMap;

/// Sorts charts based on their dependencies using topological sort.
/// Returns a vector of references to charts in the order they should be deployed.
pub fn sort_charts(charts: &[Chart]) -> Result<Vec<&Chart>> {
    let mut dag = Dag::<&Chart, ()>::new();
    let mut node_map: HashMap<String, NodeIndex> = HashMap::new();

    // 1. Add all nodes to the DAG
    for chart in charts {
        let node_index = dag.add_node(chart);
        node_map.insert(chart.name.clone(), node_index);
    }

    // 2. Add edges based on dependencies
    for chart in charts {
        if let Some(deps) = &chart.depends {
            let child_node = node_map[&chart.name]; // Must exist, we just added it

            for dep_name in deps {
                let parent_node = node_map.get(dep_name).ok_or_else(|| {
                    anyhow!(
                        "Chart '{}' depends on unknown chart '{}'",
                        chart.name,
                        dep_name
                    )
                })?;

                // Add edge: parent -> child
                dag.add_edge(*parent_node, child_node, ())
                    .map_err(|WouldCycle(_)| {
                        anyhow!(
                            "Circular dependency detected involving chart '{}'",
                            chart.name
                        )
                    })?;
            }
        }
    }

    // 3. Topological sort
    // daggy uses petgraph underneath. We can use petgraph's toposort.
    // However, daggy doesn't expose `algo::toposort` directly on `Dag` easily?
    // Let's check if we can access the underlying graph or use `daggy` features.
    // `daggy`'s main feature is strict DAG construction, so `petgraph::algo::toposort`
    // should work if we can access the graph.
    // `Dag` implements `Into<Graph>` or similar? Or `graph()`.

    // Actually, `petgraph::algo::toposort` takes `&Graph`.
    // `daggy::Dag` wraps `petgraph::Graph`.
    // Documentation says we can get the underlying graph with `.graph()`.

    let sorted_charts = petgraph::algo::toposort(dag.graph(), None)
        .map_err(|e| anyhow!("Cycle detected in chart dependencies: {:?}", e))
        .map(|indices| {
            indices
                .into_iter()
                .map(|idx| *dag.node_weight(idx).expect("Node weight should exist"))
                .collect()
        })?;

    Ok(sorted_charts)
}

/// Returns a list of charts that depend on the given target chart.
pub fn get_dependents<'a>(charts: &'a [Chart], target_name: &str) -> Result<Vec<&'a Chart>> {
    let mut dag = Dag::<&Chart, ()>::new();
    let mut node_map: HashMap<String, NodeIndex> = HashMap::new();

    // 1. Add all nodes to the DAG
    for chart in charts {
        let node_index = dag.add_node(chart);
        node_map.insert(chart.name.clone(), node_index);
    }

    // 2. Add edges based on dependencies (parent -> child)
    for chart in charts {
        if let Some(deps) = &chart.depends {
            let child_node = node_map[&chart.name];

            for dep_name in deps {
                if let Some(parent_node) = node_map.get(dep_name) {
                    // Add edge: dependent_on -> dependent
                    dag.add_edge(*parent_node, child_node, ())
                        .map_err(|WouldCycle(_)| {
                            anyhow!(
                                "Circular dependency detected involving chart '{}'",
                                chart.name
                            )
                        })?;
                }
                // If dependency is missing, we can ignore it here as we only care about existing relationships
                // or we could error. sort_charts errors. Let's error to be safe.
                else {
                    return Err(anyhow!(
                        "Chart '{}' depends on unknown chart '{}'",
                        chart.name,
                        dep_name
                    ));
                }
            }
        }
    }

    // 3. Find target node
    let target_node = node_map
        .get(target_name)
        .ok_or_else(|| anyhow!("Chart '{}' not found", target_name))?;

    // 4. Get dependents (children)
    // children() returns an iterator of (EdgeIndex, NodeIndex)
    let dependents: Vec<&Chart> = dag
        .children(*target_node)
        .iter(&dag)
        .map(|(_, node_idx)| {
            dag.node_weight(node_idx)
                .ok_or_else(|| anyhow!("Node weight missing"))
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .copied()
        .collect();

    Ok(dependents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Chart;

    fn create_chart(name: &str, depends: Option<Vec<String>>) -> Chart {
        Chart {
            name: name.to_string(),
            repo_name: Some("test".to_string()),
            version: Some("1.0.0".to_string()),
            namespace: "default".to_string(),
            dest: None,
            chart_path: None,
            no_sync: false,
            no_deploy: false,
            comment: None,
            values_files: None,
            helm_args_append: None,
            helm_args_override: None,
            values: None,
            depends,
            no_interpolation: false,
        }
    }

    #[test]
    fn test_sort_no_deps() {
        let charts = vec![create_chart("a", None), create_chart("b", None)];
        let sorted = sort_charts(&charts).unwrap();
        // Toposort order is not strictly deterministic for independent nodes unless we constrain it,
        // but it should contain all nodes.
        assert_eq!(sorted.len(), 2);
        let names: Vec<&str> = sorted.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"a"));
        assert!(names.contains(&"b"));
    }

    #[test]
    fn test_sort_simple_dep() {
        let charts = vec![
            create_chart("b", Some(vec!["a".to_string()])),
            create_chart("a", None),
        ];
        let sorted = sort_charts(&charts).unwrap();
        assert_eq!(sorted[0].name, "a");
        assert_eq!(sorted[1].name, "b");
    }

    #[test]
    fn test_sort_chain() {
        let charts = vec![
            create_chart("c", Some(vec!["b".to_string()])),
            create_chart("a", None),
            create_chart("b", Some(vec!["a".to_string()])),
        ];
        let sorted = sort_charts(&charts).unwrap();
        // a -> b -> c
        assert_eq!(sorted[0].name, "a");
        assert_eq!(sorted[1].name, "b");
        assert_eq!(sorted[2].name, "c");
    }

    #[test]
    fn test_cycle() {
        let charts = vec![
            create_chart("a", Some(vec!["b".to_string()])),
            create_chart("b", Some(vec!["a".to_string()])),
        ];
        assert!(sort_charts(&charts).is_err());
    }

    #[test]
    fn test_missing_dep() {
        let charts = vec![create_chart("a", Some(vec!["full-missing".to_string()]))];
        assert!(sort_charts(&charts).is_err());
    }

    #[test]
    fn test_get_dependents() {
        let charts = vec![
            create_chart("root", None),
            create_chart("child1", Some(vec!["root".to_string()])),
            create_chart("child2", Some(vec!["root".to_string()])),
            create_chart("grandchild", Some(vec!["child1".to_string()])),
        ];

        // Dependents of root should be child1 and child2
        let deps_root = get_dependents(&charts, "root").unwrap();
        assert_eq!(deps_root.len(), 2);
        let names_root: Vec<&str> = deps_root.iter().map(|c| c.name.as_str()).collect();
        assert!(names_root.contains(&"child1"));
        assert!(names_root.contains(&"child2"));

        // Dependents of child1 should be grandchild
        let deps_child1 = get_dependents(&charts, "child1").unwrap();
        assert_eq!(deps_child1.len(), 1);
        assert_eq!(deps_child1[0].name, "grandchild");

        // Dependents of grandchild should be empty
        let deps_grandchild = get_dependents(&charts, "grandchild").unwrap();
        assert!(deps_grandchild.is_empty());
    }
}
