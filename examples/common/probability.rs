pub fn softmax(logits: &[f32]) -> Vec<f32> {
    assert!(!logits.is_empty(), "a softmax weighs no class");
    let highest = logits.iter().copied().fold(f32::MIN, f32::max);
    let weights = logits
        .iter()
        .map(|logit| (logit - highest).exp())
        .collect::<Vec<f32>>();
    let total = weights.iter().sum::<f32>();
    weights.into_iter().map(|weight| weight / total).collect()
}
