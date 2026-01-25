// Lexi Wars Rule System
//
// Rules are cycled sequentially (not random). After all rules have been used,
// the cycle restarts with increased minimum word length.

use serde::{Deserialize, Serialize};

/// Context for rule validation - acts as difficulty settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleContext {
    pub min_word_length: usize,
    pub random_letter: char,
    pub round_number: usize,
    pub rule_index: usize,
}

impl RuleContext {
    pub fn new(round_number: usize, rule_index: usize, min_word_length: usize) -> Self {
        let random_letter = Self::generate_random_letter();

        Self {
            min_word_length,
            random_letter,
            round_number,
            rule_index,
        }
    }

    fn generate_random_letter() -> char {
        use rand::Rng;
        // Common letters weighted more heavily for fairness
        const LETTERS: &[char] = &[
            'a', 'a', 'e', 'e', 'i', 'i', 'o', 'o', 'u', 'b', 'c', 'd', 'f', 'g', 'h', 'l', 'm',
            'n', 'p', 'r', 's', 't', 'w',
        ];
        let idx = rand::rng().random_range(0..LETTERS.len());
        LETTERS[idx]
    }

    /// Regenerate the random letter for a new turn
    pub fn regenerate_letter(&mut self) {
        self.random_letter = Self::generate_random_letter();
    }
}

/// A game rule that players must follow
#[derive(Debug, Clone)]
pub struct Rule {
    pub name: String,
    pub description: String,
    pub validate: fn(&str, &RuleContext) -> Result<(), String>,
}

impl Rule {
    /// Serialize rule for sending to clients (without the validate function)
    pub fn to_client_rule(&self) -> ClientRule {
        ClientRule {
            name: self.name.clone(),
            description: self.description.clone(),
        }
    }
}

/// Client-safe representation of a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientRule {
    pub name: String,
    pub description: String,
}

/// Generate available rules for the game (order matters - cycled sequentially)
pub fn lexi_wars_rules(ctx: &RuleContext) -> Vec<Rule> {
    vec![
        Rule {
            name: "min_length".to_string(),
            description: format!(
                "Word must be at least {} characters!",
                ctx.min_word_length
            ),
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
                let word_lower = word.to_lowercase();
                if !word_lower.contains(ctx.random_letter) {
                    Err(format!("Word must contain '{}'", ctx.random_letter))
                } else if word.len() < ctx.min_word_length {
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
            name: "starts_with".to_string(),
            description: format!(
                "Word must start with '{}' and be at least {} characters long",
                ctx.random_letter, ctx.min_word_length
            ),
            validate: |word, ctx| {
                let word_lower = word.to_lowercase();
                if !word_lower.starts_with(ctx.random_letter) {
                    Err(format!("Word must start with '{}'", ctx.random_letter))
                } else if word.len() < ctx.min_word_length {
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
            name: "ends_with".to_string(),
            description: format!(
                "Word must end with '{}' and be at least {} characters long",
                ctx.random_letter, ctx.min_word_length
            ),
            validate: |word, ctx| {
                let word_lower = word.to_lowercase();
                if !word_lower.ends_with(ctx.random_letter) {
                    Err(format!("Word must end with '{}'", ctx.random_letter))
                } else if word.len() < ctx.min_word_length {
                    Err(format!(
                        "Word must be at least {} characters!",
                        ctx.min_word_length
                    ))
                } else {
                    Ok(())
                }
            },
        },
    ]
}

/// Get the total number of rules available
pub fn rule_count() -> usize {
    4 // We have 4 rules
}

/// Get rule at a specific index with the given context
pub fn get_rule_at_index(ctx: &RuleContext) -> Rule {
    let rules = lexi_wars_rules(ctx);
    let index = ctx.rule_index % rules.len();
    rules[index].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_context_creation() {
        let ctx = RuleContext::new(1, 0, 4);
        assert_eq!(ctx.min_word_length, 4);
        assert_eq!(ctx.round_number, 1);
        assert_eq!(ctx.rule_index, 0);
    }

    #[test]
    fn test_rule_validation() {
        let ctx = RuleContext {
            min_word_length: 4,
            random_letter: 'a',
            round_number: 1,
            rule_index: 0,
        };

        let rules = lexi_wars_rules(&ctx);

        // Test min_length rule
        let min_length_rule = &rules[0];
        assert!((min_length_rule.validate)("test", &ctx).is_ok());
        assert!((min_length_rule.validate)("hi", &ctx).is_err());

        // Test contains_letter rule
        let contains_rule = &rules[1];
        assert!((contains_rule.validate)("apple", &ctx).is_ok());
        assert!((contains_rule.validate)("test", &ctx).is_err()); // no 'a'
    }

    #[test]
    fn test_rule_cycling() {
        let ctx = RuleContext::new(1, 0, 4);
        let rule0 = get_rule_at_index(&ctx);
        assert_eq!(rule0.name, "min_length");

        let ctx = RuleContext::new(1, 1, 4);
        let rule1 = get_rule_at_index(&ctx);
        assert_eq!(rule1.name, "contains_letter");

        // After 4 rules, should wrap around
        let ctx = RuleContext::new(1, 4, 4);
        let rule4 = get_rule_at_index(&ctx);
        assert_eq!(rule4.name, "min_length");
    }
}
