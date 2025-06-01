use std::collections::HashSet;

use crate::package::Derivation;

/// filters any derivation by if it has at least one enabled tag and no disabled tags or no enabled and no disabled tags, if exclusive=false then
fn filter_derivations_by_tags(
    derivations: Vec<Derivation>,
    enabled_tags: Vec<String>,
    disabled_tags: Vec<String>,
    exclusive: bool,
) -> Vec<Derivation> {
    let mut new_derivations = Vec::<Derivation>::new();
    'root: for derivation in derivations {
        // moving derivation into loop
        for enabled in &enabled_tags {
            if derivation.tags.contains(enabled) {
                if disabled_tags.iter().any(|d| derivation.tags.contains(d)) {
                    continue 'root;
                }
                new_derivations.push(derivation);
                continue 'root;
            }
        }
        if !exclusive {
            new_derivations.push(derivation);
        }
    }
    new_derivations
}

fn dedup(derivations: Vec<Derivation>) -> Vec<Derivation> {
    let mut tmp = HashSet::<Derivation>::new();
    for derivation in derivations {
        tmp.insert(derivation);
    }
    tmp.into_iter().collect()
}
