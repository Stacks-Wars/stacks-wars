use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleContext {
    pub min_word_length: usize,
    pub random_letter: char,
}

#[derive(Clone)]
pub struct Rule {
    pub name: String,
    pub description: String,
    pub validate: fn(&str, &RuleContext) -> Result<(), String>,
}

pub fn get_rules(ctx: &RuleContext) -> Vec<Rule> {
    vec![
        Rule {
            name: "min_length".to_string(),
            description: format!("Word must be at least {} characters!", ctx.min_word_length),
            validate: |word, ctx| {
                if word.len() < ctx.min_word_length {
                    Err(format!(
                        "Word must be at least {} characters!",
                        ctx.min_word_length
                    ))
                } else {
                    Ok(())
                }
            },
        },
        Rule {
            name: "contains_letter".to_string(),
            description: format!(
                "Word must contain the letter '{}' and be at least {} characters long",
                ctx.random_letter, ctx.min_word_length
            ),
            validate: |word, ctx| {
                if !word.contains(ctx.random_letter) {
                    Err(format!("Word must contain '{}'", ctx.random_letter))
                } else {
                    Ok(())
                }
            },
        },
        Rule {
            name: "not_contains_letter".to_string(),
            description: format!(
                "Word must NOT contain the letter '{}' and be at least {} characters long",
                ctx.random_letter, ctx.min_word_length
            ),
            validate: |word, ctx| {
                if word.contains(ctx.random_letter) {
                    Err(format!("Word must NOT contain '{}'", ctx.random_letter))
                } else {
                    Ok(())
                }
            },
        },
        Rule {
            name: "starts_with_letter".to_string(),
            description: format!(
                "Word must start with the letter '{}' and be at least {} characters long",
                ctx.random_letter, ctx.min_word_length
            ),
            validate: |word, ctx| {
                if !word.starts_with(ctx.random_letter) {
                    Err(format!("Word must start with '{}'", ctx.random_letter))
                } else {
                    Ok(())
                }
            },
        },
        Rule {
            name: "ends_with_letter".to_string(),
            description: format!(
                "Word must end with the letter '{}' and be at least {} characters long",
                ctx.random_letter, ctx.min_word_length
            ),
            validate: |word, ctx| {
                if !word.ends_with(ctx.random_letter) {
                    Err(format!("Word must end with '{}'", ctx.random_letter))
                } else {
                    Ok(())
                }
            },
        },
        Rule {
            name: "ends_with_tion".to_string(),
            description: format!(
                "Word must end with 'tion' and be at least {} characters long",
                ctx.min_word_length
            ),
            validate: |word, _ctx| {
                if !word.ends_with("tion") {
                    Err("Word must end with 'tion'".to_string())
                } else {
                    Ok(())
                }
            },
        },
        Rule {
            name: "starts_with_co".to_string(),
            description: format!(
                "Word must start with 'co' and be at least {} characters long",
                ctx.min_word_length
            ),
            validate: |word, _ctx| {
                if !word.starts_with("co") {
                    Err("Word must start with 'co'".to_string())
                } else {
                    Ok(())
                }
            },
        },
        Rule {
            name: "double_letters".to_string(),
            description: format!(
                "Word must contain at least two pairs of double letters and be at least {} characters long",
                ctx.min_word_length
            ),
            validate: |word, _ctx| {
                let chars: Vec<char> = word.chars().collect();
                let mut double_letter_count = 0;
                let mut i = 0;

                while i < chars.len() - 1 {
                    if chars[i] == chars[i + 1] {
                        double_letter_count += 1;
                        i += 2; // Skip the next character to avoid overlapping pairs
                    } else {
                        i += 1;
                    }
                }

                if double_letter_count < 2 {
                    Err("Word must have at least two pairs of double letters!".to_string())
                } else {
                    Ok(())
                }
            },
        },
        Rule {
            name: "exact_length".to_string(),
            description: format!("Word must have exactly {} letters", ctx.min_word_length + 2),
            validate: |word, ctx| {
                let target_length = ctx.min_word_length + 2;
                if word.len() != target_length {
                    Err(format!(
                        "Word must be exactly {} letters long!",
                        target_length
                    ))
                } else {
                    Ok(())
                }
            },
        },
        Rule {
            name: "consonant_start_end".to_string(),
            description: format!(
                "Word must start and end with a consonant and be at least {} characters long",
                ctx.min_word_length
            ),
            validate: |word, _ctx| {
                let chars: Vec<char> = word.to_lowercase().chars().collect();
                if chars.is_empty() {
                    return Err("Word cannot be empty".to_string());
                }

                let vowels = "aeiou";
                let first_is_consonant = !vowels.contains(chars[0]);
                let last_is_consonant = !vowels.contains(chars[chars.len() - 1]);

                if !first_is_consonant || !last_is_consonant {
                    Err("Word must start and end with a consonant!".to_string())
                } else {
                    Ok(())
                }
            },
        },
        Rule {
            name: "vowel_start_end".to_string(),
            description: format!(
                "Word must start and end with a vowel and be at least {} characters long",
                ctx.min_word_length
            ),
            validate: |word, _ctx| {
                let chars: Vec<char> = word.to_lowercase().chars().collect();
                if chars.is_empty() {
                    return Err("Word cannot be empty".to_string());
                }

                let vowels = "aeiou";
                let first_is_vowel = vowels.contains(chars[0]);
                let last_is_vowel = vowels.contains(chars[chars.len() - 1]);

                if !first_is_vowel || !last_is_vowel {
                    Err("Word must start and end with a vowel!".to_string())
                } else {
                    Ok(())
                }
            },
        },
        Rule {
            name: "triple_letter".to_string(),
            description: format!(
                "Word must contain at least one letter that appears exactly three times and be at least {} characters long",
                ctx.min_word_length
            ),
            validate: |word, _ctx| {
                let mut letter_counts: HashMap<char, usize> = HashMap::new();
                for ch in word.chars() {
                    *letter_counts.entry(ch).or_insert(0) += 1;
                }

                let has_triple_letter = letter_counts.values().any(|&count| count == 3);
                if !has_triple_letter {
                    Err(
                        "Word must contain at least one letter appearing exactly three times!"
                            .to_string(),
                    )
                } else {
                    Ok(())
                }
            },
        },
        Rule {
            name: "palindrome".to_string(),
            description: format!(
                "Word must be a palindrome and be at least {} characters long",
                ctx.min_word_length
            ),
            validate: |word, _ctx| {
                let reversed: String = word.chars().rev().collect();
                if word != reversed {
                    Err("Word must be a palindrome".to_string())
                } else {
                    Ok(())
                }
            },
        },
        Rule {
            name: "no_repeating_letters".to_string(),
            description: format!(
                "Word must have no repeating letters and be at least {} characters long",
                ctx.min_word_length
            ),
            validate: |word, _ctx| {
                let unique_chars: std::collections::HashSet<char> = word.chars().collect();
                if unique_chars.len() != word.len() {
                    Err("Word must have no repeating letters!".to_string())
                } else {
                    Ok(())
                }
            },
        },
        Rule {
            name: "exact_vowels_consonants".to_string(),
            description: format!(
                "Word must contain exactly 3 vowels and 3 consonants and be at least {} characters long",
                ctx.min_word_length
            ),
            validate: |word, _ctx| {
                let vowels = "aeiou";
                let mut vowel_count = 0;
                let mut consonant_count = 0;

                for ch in word.to_lowercase().chars() {
                    if ch.is_alphabetic() {
                        if vowels.contains(ch) {
                            vowel_count += 1;
                        } else {
                            consonant_count += 1;
                        }
                    }
                }

                if vowel_count != 3 || consonant_count != 3 {
                    Err("Word must contain exactly 3 vowels and 3 consonants!".to_string())
                } else {
                    Ok(())
                }
            },
        },
        Rule {
            name: "same_letter_three_times".to_string(),
            description: format!(
                "Word must contain the same letter three times and be at least {} characters long",
                ctx.min_word_length
            ),
            validate: |word, _ctx| {
                let mut letter_counts: HashMap<char, usize> = HashMap::new();
                for ch in word.chars() {
                    *letter_counts.entry(ch).or_insert(0) += 1;
                }

                if !letter_counts.values().any(|&count| count >= 3) {
                    Err("Word must contain the same letter at least three times!".to_string())
                } else {
                    Ok(())
                }
            },
        },
        Rule {
            name: "equal_vowels_consonants".to_string(),
            description: format!(
                "Word must have an equal number of vowels and consonants and be at least {} characters long",
                ctx.min_word_length
            ),
            validate: |word, _ctx| {
                let vowels = "aeiou";
                let mut vowel_count = 0;
                let mut consonant_count = 0;

                for ch in word.to_lowercase().chars() {
                    if ch.is_alphabetic() {
                        if vowels.contains(ch) {
                            vowel_count += 1;
                        } else {
                            consonant_count += 1;
                        }
                    }
                }

                if vowel_count != consonant_count {
                    Err("Word must have an equal number of vowels and consonants!".to_string())
                } else {
                    Ok(())
                }
            },
        },
    ]
}

pub fn find_rule_by_name<'a>(rules: &'a [Rule], name: &str) -> Option<&'a Rule> {
    rules.iter().find(|rule| rule.name == name)
}

pub fn get_rule_by_index(index: usize, ctx: &RuleContext) -> Option<Rule> {
    get_rules(ctx).get(index).cloned()
}
