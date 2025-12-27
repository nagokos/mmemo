use std::path::Path;

#[derive(Debug)]
pub struct Matcher {
    targets: Vec<String>,
}

impl Matcher {
    pub fn new(targets: Vec<String>) -> Self {
        Matcher { targets }
    }
    pub fn fuzzy_match(&self, input: &str) -> Vec<MatchResult> {
        self.targets
            .iter()
            .filter_map(|target| {
                let target = Path::new(target)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(target);

                // matchで表現できるようにするのはどうか
                // fn matchkindみたいなの作ってそれで条件分岐
                if input == target {
                    // Exact
                    Some(MatchResult {
                        target: target.to_owned(),
                        kind: MatchKind::Exact,
                        score: None,
                    })
                } else if target.starts_with(&input[..input.len()]) {
                    // Prefix
                    Some(MatchResult {
                        target: target.to_owned(),
                        kind: MatchKind::Prefix,
                        score: None,
                    })
                } else if target.contains(input) {
                    // Substring
                    let input_char = input.chars().next()?;
                    let distance = target
                        .chars()
                        .position(|target_char| target_char == input_char)?;

                    Some(MatchResult {
                        target: target.to_owned(),
                        kind: MatchKind::Substring,
                        // 小さい方が表示で有利
                        score: Some(distance),
                    })
                } else {
                    // Fuzzy
                    let mut index = 0;
                    let mut score = 0;
                    let mut previous_hit = false;

                    'input: for input_char in input.chars() {
                        for target_char in target.chars().skip(index) {
                            index += 1;

                            if input_char == target_char {
                                if previous_hit {
                                    score += 5;
                                } else {
                                    previous_hit = true;
                                    score += 3;
                                }
                                continue 'input;
                            }

                            previous_hit = false
                        }
                        return None;
                    }

                    Some(MatchResult {
                        target: target.to_owned(),
                        kind: MatchKind::Fuzzy,
                        score: Some(score),
                    })
                }
            })
            .collect::<Vec<_>>()
    }
}

#[derive(Debug)]
pub struct MatchResult {
    target: String,
    kind: MatchKind,
    score: Option<usize>,
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
enum MatchKind {
    // "abc" "abc.md"
    Exact,
    // "abc" "abc_memo.md"
    Prefix,
    // "abc" "my_abc_memo.md"
    Substring,
    // "abc" "a_b_c.md"
    Fuzzy,
}
