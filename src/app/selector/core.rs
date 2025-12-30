#[derive(Debug)]
pub struct Matcher {
    pub items: Vec<String>,
    cache: Option<(String, Vec<usize>)>,
}

impl Matcher {
    pub fn new(items: Vec<String>) -> Self {
        Matcher { items, cache: None }
    }

    pub fn fuzzy_match(&mut self, input: &str) -> Vec<MatchResult> {
        if let Some((prev, indices)) = &self.cache {
            if input.starts_with(prev) && indices.is_empty() {
                return Vec::new();
            }
        }

        if input.is_empty() {
            self.cache = None;
            return self
                .items
                .iter()
                .map(|item| MatchResult {
                    item: item.to_string(),
                    kind: MatchKind::Unfiltered,
                    score: None,
                    hits: Vec::new(),
                })
                .collect();
        }

        // 前回の入力で始まるなら、キャッシュした候補だけ検索
        let search_indices: Vec<usize> = match &self.cache {
            Some((prev, indices)) if input.starts_with(prev) => indices.clone(),
            _ => (0..self.items.len()).collect(),
        };

        let mut matched_indices = Vec::new();
        let mut result = Vec::new();

        for i in search_indices {
            let item = &self.items[i];
            if let Some(m) = self.match_item(item, input) {
                matched_indices.push(i);
                result.push(m);
            }
        }

        self.cache = Some((input.to_string(), matched_indices));

        result.sort_by(|a, b| (a.kind, a.score).cmp(&(b.kind, b.score)));
        result
    }

    fn match_item(&self, item: &str, input: &str) -> Option<MatchResult> {
        if input == item {
            Some(MatchResult {
                item: item.to_owned(),
                kind: MatchKind::Exact,
                score: None,
                hits: (0..input.len()).collect(),
            })
        } else if item.starts_with(input) {
            Some(MatchResult {
                item: item.to_owned(),
                kind: MatchKind::Prefix,
                score: None,
                hits: (0..input.len()).collect(),
            })
        } else if item.contains(input) {
            let input_char = input.chars().next()?;
            let distance = item
                .chars()
                .position(|target_char| target_char == input_char)?;
            Some(MatchResult {
                item: item.to_owned(),
                kind: MatchKind::Substring,
                score: Some(distance),
                hits: (distance..distance + input.len()).collect(),
            })
        } else {
            // Fuzzy
            let mut index = 0;
            let mut score = 0;
            let mut previous_hit = false;
            let mut hits = Vec::new();
            'input: for input_char in input.chars() {
                for target_char in item.chars().skip(index) {
                    index += 1;
                    if input_char == target_char {
                        hits.push(index - 1);
                        if previous_hit {
                            score += 5;
                        } else {
                            previous_hit = true;
                            score += 3;
                        }
                        continue 'input;
                    } else {
                        previous_hit = false
                    }
                }
                return None;
            }
            Some(MatchResult {
                item: item.to_owned(),
                kind: MatchKind::Fuzzy,
                score: Some(score),
                hits,
            })
        }
    }
}

#[derive(Debug)]
pub struct MatchResult {
    pub item: String,
    kind: MatchKind,
    score: Option<usize>,
    pub hits: Vec<usize>,
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
enum MatchKind {
    Exact,
    Prefix,
    Substring,
    Fuzzy,
    Unfiltered,
}
