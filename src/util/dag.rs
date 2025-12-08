use crate::config::Chart;
use crate::util::error::DagError;
use daggy::{Dag, NodeIndex, Walker, WouldCycle};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

/// Sorts charts based on their dependencies using topological sort.
/// Returns a vector of references to charts in the order they should be deployed.
pub fn sort_charts(charts: &[Chart]) -> Result<Vec<&Chart>, DagError> {
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
                    DagError::UnknownDependency(chart.name.clone(), dep_name.clone())
                })?;

                // Add edge: parent -> child
                dag.add_edge(*parent_node, child_node, ())
                    .map_err(|WouldCycle(_)| DagError::CircularDependency(chart.name.clone()))?;
            }
        }
    }

    // 3. Stable Topological Sort (Kahn's Algorithm)
    // We strive to respect the original configuration order for independent nodes.
    // Since we added nodes in order, their NodeIndex corresponds to their config order.
    // We use a PriorityQueue (MinHeap) to always pick the available node with the smallest NodeIndex.

    let graph = dag.graph();
    let mut in_degree = HashMap::new();
    let mut queue = BinaryHeap::new();

    // Initialize in-degrees
    for node_idx in graph.node_indices() {
        let count = graph
            .neighbors_directed(node_idx, petgraph::Direction::Incoming)
            .count();
        in_degree.insert(node_idx, count);
        if count == 0 {
            queue.push(Reverse(node_idx));
        }
    }

    let mut sorted_indices = Vec::new();
    while let Some(Reverse(node_idx)) = queue.pop() {
        sorted_indices.push(node_idx);

        // For each child
        for child_idx in graph.neighbors_directed(node_idx, petgraph::Direction::Outgoing) {
            let entry = in_degree.get_mut(&child_idx).ok_or_else(|| {
                DagError::InternalError("In-degree entry missing for known child node".to_string())
            })?;
            *entry -= 1;
            if *entry == 0 {
                queue.push(Reverse(child_idx));
            }
        }
    }

    if sorted_indices.len() != graph.node_count() {
        // This should have been caught by WouldCycle check during construction
        unreachable!("Cycle detected in DAG despite WouldCycle checks");
    }

    let sorted_charts = sorted_indices
        .into_iter()
        .map(|idx| {
            dag.node_weight(idx)
                .ok_or(DagError::NodeWeightMissing)
                .copied()
        })
        .collect::<Result<Vec<_>, DagError>>()?;

    Ok(sorted_charts)
}

/// Returns a list of charts that depend on the given target chart.
pub fn get_dependents<'a>(
    charts: &'a [Chart],
    target_name: &str,
) -> Result<Vec<&'a Chart>, DagError> {
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
                            DagError::CircularDependency(chart.name.clone())
                        })?;
                }
                // If dependency is missing, we can ignore it here as we only care about existing relationships
                // or we could error. sort_charts errors. Let's error to be safe.
                else {
                    return Err(DagError::UnknownDependency(
                        chart.name.clone(),
                        dep_name.clone(),
                    ));
                }
            }
        }
    }

    // 3. Find target node
    let target_node = node_map
        .get(target_name)
        .ok_or_else(|| DagError::ChartNotFound(target_name.to_string()))?;

    // 4. Get dependents (children)
    // children() returns an iterator of (EdgeIndex, NodeIndex)
    let dependents: Vec<&Chart> = dag
        .children(*target_node)
        .iter(&dag)
        .map(|(_, node_idx)| dag.node_weight(node_idx).ok_or(DagError::NodeWeightMissing))
        .collect::<Result<Vec<_>, DagError>>()?
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
            release_name: None,
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

    #[test]
    fn test_sort_complex_dag() {
        // A -> B -> C
        // A -> D
        // E (independent)
        let charts = vec![
            create_chart("c", Some(vec!["b".to_string()])),
            create_chart("d", Some(vec!["a".to_string()])),
            create_chart("a", None),
            create_chart("b", Some(vec!["a".to_string()])),
            create_chart("e", None),
        ];
        let sorted = sort_charts(&charts).unwrap();
        let names: Vec<String> = sorted.iter().map(|c| c.name.clone()).collect();

        // Verify dependencies
        let idx_a = names.iter().position(|r| r == "a").unwrap();
        let idx_b = names.iter().position(|r| r == "b").unwrap();
        let idx_c = names.iter().position(|r| r == "c").unwrap();
        let idx_d = names.iter().position(|r| r == "d").unwrap();

        assert!(idx_a < idx_b, "a must be before b");
        assert!(idx_b < idx_c, "b must be before c");
        assert!(idx_a < idx_d, "a must be before d");
    }

    #[test]
    fn test_sort_stability_independent() {
        // Case 1: z, a, m
        let charts1 = vec![
            create_chart("z", None),
            create_chart("a", None),
            create_chart("m", None),
        ];
        let sorted1 = sort_charts(&charts1).unwrap();
        let names1: Vec<String> = sorted1.iter().map(|c| c.name.clone()).collect();

        // Case 2: m, a, z (reverse input)
        let charts2 = vec![
            create_chart("m", None),
            create_chart("a", None),
            create_chart("z", None),
        ];
        let sorted2 = sort_charts(&charts2).unwrap();
        let names2: Vec<String> = sorted2.iter().map(|c| c.name.clone()).collect();

        // If we want to respect config order, names2 should be ["m", "a", "z"]
        assert_eq!(names1, vec!["z", "a", "m"]);
        assert_eq!(names2, vec!["m", "a", "z"]);
    }
}
