use rand::Rng;

const PREFIXES: &[&str] = &[
    "Al", "Bran", "Dar", "El", "Gar", "Mal", "Ser", "Tor", "Val", "Wyn",
];
const SUFFIXES: &[&str] = &[
    "a", "dor", "en", "ian", "is", "mir", "on", "ric", "us", "wen",
];

/// Генерирует случайное фэнтезийное имя героя, не длиннее 20 символов.
pub fn generate(rng: &mut impl Rng) -> String {
    let prefix = PREFIXES[rng.gen_range(0..PREFIXES.len())];
    let suffix = SUFFIXES[rng.gen_range(0..SUFFIXES.len())];
    let name = format!("{prefix}{suffix}");
    name.chars().take(20).collect()
}

// ---------------------------------------------------------------------------
// Тесты
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn generate_returns_nonempty() {
        let mut rng = StdRng::seed_from_u64(0);
        let name = generate(&mut rng);
        assert!(!name.is_empty());
    }

    #[test]
    fn generate_respects_length_limit() {
        let mut rng = StdRng::seed_from_u64(0);
        for _ in 0..100 {
            let name = generate(&mut rng);
            assert!(name.len() <= 20, "name '{name}' exceeds 20 chars");
        }
    }

    #[test]
    fn generate_is_deterministic() {
        let mut rng1 = StdRng::seed_from_u64(42);
        let mut rng2 = StdRng::seed_from_u64(42);
        assert_eq!(generate(&mut rng1), generate(&mut rng2));
    }
}
