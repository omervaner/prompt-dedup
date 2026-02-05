use std::collections::{HashSet, HashMap};

/// Calculate Jaccard similarity between two strings based on words
pub fn jaccard_similarity(a: &str, b: &str) -> f32 {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();

    let words_a: HashSet<&str> = a_lower.split_whitespace().collect();
    let words_b: HashSet<&str> = b_lower.split_whitespace().collect();

    if words_a.is_empty() && words_b.is_empty() {
        return 1.0;
    }

    let intersection = words_a.intersection(&words_b).count();
    let union = words_a.union(&words_b).count();

    if union == 0 {
        return 0.0;
    }

    intersection as f32 / union as f32
}

/// A pair of similar prompts
#[derive(Clone)]
pub struct SimilarPair {
    pub id_a: i64,
    pub text_a: String,
    pub id_b: i64,
    pub text_b: String,
    pub similarity: f32,
}

/// Find all pairs of prompts above the similarity threshold
pub fn find_similar_pairs(
    prompts: &[(i64, String)],
    threshold: f32,
) -> Vec<SimilarPair> {
    let mut pairs = Vec::new();

    for i in 0..prompts.len() {
        for j in (i + 1)..prompts.len() {
            let sim = jaccard_similarity(&prompts[i].1, &prompts[j].1);
            if sim >= threshold {
                pairs.push(SimilarPair {
                    id_a: prompts[i].0,
                    text_a: prompts[i].1.clone(),
                    id_b: prompts[j].0,
                    text_b: prompts[j].1.clone(),
                    similarity: sim,
                });
            }
        }
    }

    // Sort by similarity descending
    pairs.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
    pairs
}

/// Group similar prompts together (transitive grouping)
pub fn group_similar_prompts(
    prompts: &[(i64, String)],
    threshold: f32,
) -> Vec<Vec<(i64, String, f32)>> {
    // Build adjacency map
    let mut adjacency: HashMap<i64, Vec<(i64, f32)>> = HashMap::new();

    for i in 0..prompts.len() {
        for j in (i + 1)..prompts.len() {
            let sim = jaccard_similarity(&prompts[i].1, &prompts[j].1);
            if sim >= threshold {
                adjacency
                    .entry(prompts[i].0)
                    .or_default()
                    .push((prompts[j].0, sim));
                adjacency
                    .entry(prompts[j].0)
                    .or_default()
                    .push((prompts[i].0, sim));
            }
        }
    }

    // Find connected components using DFS
    let mut visited: HashSet<i64> = HashSet::new();
    let mut groups: Vec<Vec<(i64, String, f32)>> = Vec::new();

    let id_to_text: HashMap<i64, String> = prompts.iter().cloned().collect();

    for (id, _text) in prompts {
        if visited.contains(id) {
            continue;
        }
        if !adjacency.contains_key(id) {
            continue; // No similar prompts
        }

        let mut group = Vec::new();
        let mut stack = vec![(*id, 1.0f32)];

        while let Some((current_id, sim)) = stack.pop() {
            if visited.contains(&current_id) {
                continue;
            }
            visited.insert(current_id);

            if let Some(t) = id_to_text.get(&current_id) {
                group.push((current_id, t.clone(), sim));
            }

            if let Some(neighbors) = adjacency.get(&current_id) {
                for (neighbor_id, neighbor_sim) in neighbors {
                    if !visited.contains(neighbor_id) {
                        stack.push((*neighbor_id, *neighbor_sim));
                    }
                }
            }
        }

        if group.len() > 1 {
            groups.push(group);
        }
    }

    groups
}
