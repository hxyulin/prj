use nucleo::{
    Matcher, Utf32Str,
    pattern::{CaseMatching, Normalization, Pattern},
};

pub struct FuzzyMatcher {
    matcher: Matcher,
}

pub struct FuzzyMatch {
    pub index: usize,
    pub score: u32,
}

impl FuzzyMatcher {
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(nucleo::Config::DEFAULT),
        }
    }

    /// Filter items by query, returning matching indices sorted by score (best first).
    pub fn filter(&mut self, query: &str, items: &[String]) -> Vec<FuzzyMatch> {
        if query.is_empty() {
            return items
                .iter()
                .enumerate()
                .map(|(i, _)| FuzzyMatch { index: i, score: 0 })
                .collect();
        }

        let pattern = Pattern::new(
            query,
            CaseMatching::Smart,
            Normalization::Smart,
            nucleo::pattern::AtomKind::Fuzzy,
        );

        let mut results: Vec<FuzzyMatch> = items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| {
                let mut buf = Vec::new();
                let haystack = Utf32Str::new(item, &mut buf);
                let score = pattern.score(haystack, &mut self.matcher)?;
                Some(FuzzyMatch { index: i, score })
            })
            .collect();

        results.sort_by(|a, b| b.score.cmp(&a.score));
        results
    }
}
