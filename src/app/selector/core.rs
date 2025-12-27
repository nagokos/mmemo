#[derive(Debug)]
pub struct Matcher {
    pub items: Vec<String>,
}

impl Matcher {
    pub fn new(items: Vec<String>) -> Self {
        Matcher { items }
    }
    pub fn fuzzy_match(&self, input: &str) -> Vec<MatchResult> {
        if input.is_empty() {
            let vec = self
                .items
                .iter()
                .map(|item| MatchResult {
                    item: item.to_string(),
                    kind: MatchKind::Unfiltered,
                    score: None,
                    hits: Vec::new(),
                })
                .collect();

            return vec;
        };

        let mut result = self
            .items
            .iter()
            .filter_map(|item| {
                // matchで表現できるようにするのはどうか
                // fn matchkindみたいなの作ってそれで条件分岐
                if input == item {
                    // Exact
                    Some(MatchResult {
                        item: item.to_owned(),
                        kind: MatchKind::Exact,
                        score: None,
                        hits: (0..input.len()).collect(),
                    })
                } else if item.starts_with(input) {
                    // Prefix
                    Some(MatchResult {
                        item: item.to_owned(),
                        kind: MatchKind::Prefix,
                        score: None,
                        hits: (0..input.len()).collect(),
                    })
                } else if item.contains(input) {
                    // Substring
                    let input_char = input.chars().next()?;
                    let distance = item
                        .chars()
                        .position(|target_char| target_char == input_char)?;

                    Some(MatchResult {
                        item: item.to_owned(),
                        kind: MatchKind::Substring,
                        // 小さい方が表示で有利
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
            })
            .collect::<Vec<_>>();

        result.sort_by(|a, b| (a.kind, a.score).cmp(&(b.kind, b.score)));

        result
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
    // "abc" "abc.md"
    Exact,
    // "abc" "abc_memo.md"
    Prefix,
    // "abc" "my_abc_memo.md"
    Substring,
    // "abc" "a_b_c.md"
    Fuzzy,
    // input is empty
    Unfiltered,
}
