use stacks_wars_be::games::lexi_wars::rules::{RuleContext, find_rule_by_name, get_rules};

fn create_test_context() -> RuleContext {
    RuleContext {
        min_word_length: 4,
        random_letter: 'a',
    }
}

fn get_rule_by_name<'a>(
    rules: &'a [stacks_wars_be::games::lexi_wars::rules::Rule],
    name: &str,
) -> &'a stacks_wars_be::games::lexi_wars::rules::Rule {
    find_rule_by_name(rules, name).unwrap_or_else(|| panic!("Rule '{}' not found", name))
}

#[test]
fn test_min_length_rule() {
    let ctx = create_test_context();
    let rules = get_rules(&ctx);
    let rule = get_rule_by_name(&rules, "min_length");

    // Valid cases
    assert!((rule.validate)("hello", &ctx).is_ok());
    assert!((rule.validate)("test", &ctx).is_ok());
    assert!((rule.validate)("longer", &ctx).is_ok());

    // Invalid cases
    assert!((rule.validate)("hi", &ctx).is_err());
    assert!((rule.validate)("no", &ctx).is_err());
    assert!((rule.validate)("", &ctx).is_err());

    // Check error message
    let result = (rule.validate)("hi", &ctx);
    assert!(
        result
            .unwrap_err()
            .contains("must be at least 4 characters")
    );
}

#[test]
fn test_contains_letter_rule() {
    let ctx = create_test_context();
    let rules = get_rules(&ctx);
    let rule = get_rule_by_name(&rules, "contains_letter");

    // Valid cases
    assert!((rule.validate)("apple", &ctx).is_ok());
    assert!((rule.validate)("banana", &ctx).is_ok());
    assert!((rule.validate)("cat", &ctx).is_ok());
    assert!((rule.validate)("aardvark", &ctx).is_ok());

    // Invalid cases
    assert!((rule.validate)("hello", &ctx).is_err());
    assert!((rule.validate)("world", &ctx).is_err());
    assert!((rule.validate)("test", &ctx).is_err());

    // Check error message
    let result = (rule.validate)("hello", &ctx);
    assert!(result.unwrap_err().contains("must contain 'a'"));
}

#[test]
fn test_not_contains_letter_rule() {
    let ctx = create_test_context();
    let rules = get_rules(&ctx);
    let rule = get_rule_by_name(&rules, "not_contains_letter");

    // Valid cases
    assert!((rule.validate)("hello", &ctx).is_ok());
    assert!((rule.validate)("world", &ctx).is_ok());
    assert!((rule.validate)("test", &ctx).is_ok());
    assert!((rule.validate)("python", &ctx).is_ok());

    // Invalid cases
    assert!((rule.validate)("apple", &ctx).is_err());
    assert!((rule.validate)("banana", &ctx).is_err());
    assert!((rule.validate)("cat", &ctx).is_err());

    // Check error message
    let result = (rule.validate)("apple", &ctx);
    assert!(result.unwrap_err().contains("must NOT contain 'a'"));
}

#[test]
fn test_starts_with_letter_rule() {
    let ctx = create_test_context();
    let rules = get_rules(&ctx);
    let rule = get_rule_by_name(&rules, "starts_with_letter");

    // Valid cases
    assert!((rule.validate)("apple", &ctx).is_ok());
    assert!((rule.validate)("amazing", &ctx).is_ok());
    assert!((rule.validate)("ant", &ctx).is_ok());

    // Invalid cases
    assert!((rule.validate)("hello", &ctx).is_err());
    assert!((rule.validate)("world", &ctx).is_err());
    assert!((rule.validate)("banana", &ctx).is_err());

    // Check error message
    let result = (rule.validate)("hello", &ctx);
    assert!(result.unwrap_err().contains("must start with 'a'"));
}

#[test]
fn test_ends_with_letter_rule() {
    let ctx = create_test_context();
    let rules = get_rules(&ctx);
    let rule = get_rule_by_name(&rules, "ends_with_letter");

    // Valid cases
    assert!((rule.validate)("banana", &ctx).is_ok());
    assert!((rule.validate)("pizza", &ctx).is_ok());
    assert!((rule.validate)("area", &ctx).is_ok());

    // Invalid cases
    assert!((rule.validate)("hello", &ctx).is_err());
    assert!((rule.validate)("world", &ctx).is_err());
    assert!((rule.validate)("test", &ctx).is_err());

    // Check error message
    let result = (rule.validate)("hello", &ctx);
    assert!(result.unwrap_err().contains("must end with 'a'"));
}

#[test]
fn test_ends_with_tion_rule() {
    let ctx = create_test_context();
    let rules = get_rules(&ctx);
    let rule = get_rule_by_name(&rules, "ends_with_tion");

    // Valid cases
    assert!((rule.validate)("action", &ctx).is_ok());
    assert!((rule.validate)("nation", &ctx).is_ok());
    assert!((rule.validate)("creation", &ctx).is_ok());
    assert!((rule.validate)("education", &ctx).is_ok());

    // Invalid cases
    assert!((rule.validate)("hello", &ctx).is_err());
    assert!((rule.validate)("world", &ctx).is_err());
    assert!((rule.validate)("actions", &ctx).is_err());

    // Check error message
    let result = (rule.validate)("hello", &ctx);
    assert!(result.unwrap_err().contains("must end with 'tion'"));
}

#[test]
fn test_starts_with_co_rule() {
    let ctx = create_test_context();
    let rules = get_rules(&ctx);
    let rule = get_rule_by_name(&rules, "starts_with_co");

    // Valid cases
    assert!((rule.validate)("code", &ctx).is_ok());
    assert!((rule.validate)("computer", &ctx).is_ok());
    assert!((rule.validate)("coffee", &ctx).is_ok());
    assert!((rule.validate)("company", &ctx).is_ok());

    // Invalid cases
    assert!((rule.validate)("hello", &ctx).is_err());
    assert!((rule.validate)("world", &ctx).is_err());
    assert!((rule.validate)("ocean", &ctx).is_err());

    // Check error message
    let result = (rule.validate)("hello", &ctx);
    assert!(result.unwrap_err().contains("must start with 'co'"));
}

#[test]
fn test_double_letters_rule() {
    let ctx = create_test_context();
    let rules = get_rules(&ctx);
    let rule = get_rule_by_name(&rules, "double_letters");

    // Valid cases
    assert!((rule.validate)("bookkeeper", &ctx).is_ok()); // oo, kk, ee
    assert!((rule.validate)("balloon", &ctx).is_ok()); // ll, oo
    assert!((rule.validate)("coffee", &ctx).is_ok()); // ff, ee
    assert!((rule.validate)("success", &ctx).is_ok()); // cc, ss

    // Invalid cases - less than 2 pairs
    assert!((rule.validate)("hello", &ctx).is_err()); // only ll
    assert!((rule.validate)("world", &ctx).is_err()); // no doubles
    assert!((rule.validate)("book", &ctx).is_err()); // only oo

    // Check error message
    let result = (rule.validate)("hello", &ctx);
    assert!(
        result
            .unwrap_err()
            .contains("must have at least two pairs of double letters")
    );
}

#[test]
fn test_exact_length_rule() {
    let ctx = create_test_context();
    let rules = get_rules(&ctx);
    let rule = get_rule_by_name(&rules, "exact_length");

    // Valid cases (4 + 2 = 6 characters)
    assert!((rule.validate)("hello!", &ctx).is_ok());
    assert!((rule.validate)("abcdef", &ctx).is_ok());
    assert!((rule.validate)("python", &ctx).is_ok());

    // Invalid cases
    assert!((rule.validate)("hello", &ctx).is_err()); // 5 chars
    assert!((rule.validate)("hi", &ctx).is_err()); // 2 chars
    assert!((rule.validate)("toolong", &ctx).is_err()); // 7 chars

    // Check error message
    let result = (rule.validate)("hello", &ctx);
    assert!(
        result
            .unwrap_err()
            .contains("must be exactly 6 letters long")
    );
}

#[test]
fn test_consonant_start_end_rule() {
    let ctx = create_test_context();
    let rules = get_rules(&ctx);
    let rule = get_rule_by_name(&rules, "consonant_start_end");

    // Valid cases
    assert!((rule.validate)("python", &ctx).is_ok()); // p...n
    assert!((rule.validate)("rust", &ctx).is_ok()); // r...t
    assert!((rule.validate)("book", &ctx).is_ok()); // b...k

    // Invalid cases
    assert!((rule.validate)("apple", &ctx).is_err()); // starts with vowel
    assert!((rule.validate)("orange", &ctx).is_err()); // starts and ends with vowel
    assert!((rule.validate)("hello", &ctx).is_err()); // ends with vowel
    assert!((rule.validate)("code", &ctx).is_err()); // ends with vowel

    // Check error message
    let result = (rule.validate)("apple", &ctx);
    assert!(
        result
            .unwrap_err()
            .contains("must start and end with a consonant")
    );
}

#[test]
fn test_vowel_start_end_rule() {
    let ctx = create_test_context();
    let rules = get_rules(&ctx);
    let rule = get_rule_by_name(&rules, "vowel_start_end");

    // Valid cases
    assert!((rule.validate)("apple", &ctx).is_ok()); // a...e
    assert!((rule.validate)("orange", &ctx).is_ok()); // o...e
    assert!((rule.validate)("idea", &ctx).is_ok()); // i...a
    assert!((rule.validate)("area", &ctx).is_ok()); // a...a

    // Invalid cases
    assert!((rule.validate)("hello", &ctx).is_err()); // starts with consonant
    assert!((rule.validate)("world", &ctx).is_err()); // starts and ends with consonant
    assert!((rule.validate)("code", &ctx).is_err()); // starts with consonant

    // Check error message
    let result = (rule.validate)("hello", &ctx);
    assert!(
        result
            .unwrap_err()
            .contains("must start and end with a vowel")
    );
}

#[test]
fn test_triple_letter_rule() {
    let ctx = create_test_context();
    let rules = get_rules(&ctx);
    let rule = get_rule_by_name(&rules, "triple_letter");

    // Valid cases
    assert!((rule.validate)("aalpha", &ctx).is_ok()); // aaa
    assert!((rule.validate)("helllo", &ctx).is_ok()); // lll
    assert!((rule.validate)("tessst", &ctx).is_ok()); // sss

    // Invalid cases
    assert!((rule.validate)("hello", &ctx).is_err());
    assert!((rule.validate)("world", &ctx).is_err());
    assert!((rule.validate)("book", &ctx).is_err());
    assert!((rule.validate)("bookeper", &ctx).is_err()); // no triple letters

    // Check error message
    let result = (rule.validate)("hello", &ctx);
    assert!(
        result
            .unwrap_err()
            .contains("must contain at least one letter appearing exactly three times")
    );
}

#[test]
fn test_palindrome_rule() {
    let ctx = create_test_context();
    let rules = get_rules(&ctx);
    let rule = get_rule_by_name(&rules, "palindrome");

    // Valid cases
    assert!((rule.validate)("racecar", &ctx).is_ok());
    assert!((rule.validate)("level", &ctx).is_ok());
    assert!((rule.validate)("noon", &ctx).is_ok());
    assert!((rule.validate)("madam", &ctx).is_ok());
    assert!((rule.validate)("a", &ctx).is_ok());

    // Invalid cases
    assert!((rule.validate)("hello", &ctx).is_err());
    assert!((rule.validate)("world", &ctx).is_err());
    assert!((rule.validate)("almost", &ctx).is_err());

    // Check error message
    let result = (rule.validate)("hello", &ctx);
    assert!(result.unwrap_err().contains("must be a palindrome"));
}

#[test]
fn test_no_repeating_letters_rule() {
    let ctx = create_test_context();
    let rules = get_rules(&ctx);
    let rule = get_rule_by_name(&rules, "no_repeating_letters");

    // Valid cases
    assert!((rule.validate)("world", &ctx).is_ok());
    assert!((rule.validate)("python", &ctx).is_ok());
    assert!((rule.validate)("rust", &ctx).is_ok());
    assert!((rule.validate)("coder", &ctx).is_ok());

    // Invalid cases
    assert!((rule.validate)("hello", &ctx).is_err()); // duplicate 'l'
    assert!((rule.validate)("book", &ctx).is_err()); // duplicate 'o'
    assert!((rule.validate)("coffee", &ctx).is_err()); // duplicate 'f' and 'e'

    // Check error message
    let result = (rule.validate)("hello", &ctx);
    assert!(
        result
            .unwrap_err()
            .contains("must have no repeating letters")
    );
}

#[test]
fn test_exact_vowels_consonants_rule() {
    let ctx = create_test_context();
    let rules = get_rules(&ctx);
    let rule = get_rule_by_name(&rules, "exact_vowels_consonants");

    // Valid cases (3 vowels, 3 consonants)
    assert!((rule.validate)("louder", &ctx).is_ok()); // l-d-r (3 consonants), o-u-e (3 vowels)

    // Invalid cases
    assert!((rule.validate)("python", &ctx).is_err()); // p-y-t-h-n (5 consonants), o (1 vowel)
    assert!((rule.validate)("hello", &ctx).is_err()); // h-l-l (3 consonants), e-o (2 vowels)
    assert!((rule.validate)("world", &ctx).is_err()); // w-r-l-d (4 consonants), o (1 vowel)
    assert!((rule.validate)("mouse", &ctx).is_err()); // m-s (2 consonants), o-u-e (3 vowels)

    // Check error message
    let result = (rule.validate)("hello", &ctx);
    assert!(
        result
            .unwrap_err()
            .contains("must contain exactly 3 vowels and 3 consonants")
    );
}

#[test]
fn test_same_letter_three_times_rule() {
    let ctx = create_test_context();
    let rules = get_rules(&ctx);
    let rule = get_rule_by_name(&rules, "same_letter_three_times");

    // Valid cases
    assert!((rule.validate)("aaalpha", &ctx).is_ok()); // 'a' appears 3 times
    assert!((rule.validate)("helllo", &ctx).is_ok()); // 'l' appears 3 times
    assert!((rule.validate)("tesssst", &ctx).is_ok()); // 's' appears 3 times
    assert!((rule.validate)("banana", &ctx).is_ok()); // 'a' appears 3 times

    // Invalid cases
    assert!((rule.validate)("hello", &ctx).is_err()); // 'l' appears 2 times, others once
    assert!((rule.validate)("world", &ctx).is_err()); // all letters appear once
    assert!((rule.validate)("book", &ctx).is_err()); // 'o' appears twice

    // Check error message
    let result = (rule.validate)("hello", &ctx);
    assert!(
        result
            .unwrap_err()
            .contains("must contain the same letter at least three times!")
    );
}

#[test]
fn test_equal_vowels_consonants_rule() {
    let ctx = create_test_context();
    let rules = get_rules(&ctx);
    let rule = get_rule_by_name(&rules, "equal_vowels_consonants");

    // Valid cases
    assert!((rule.validate)("code", &ctx).is_ok()); // c-d (2 consonants), o-e (2 vowels)
    assert!((rule.validate)("bead", &ctx).is_ok()); // b-d (2 consonants), e-a (2 vowels)
    assert!((rule.validate)("poem", &ctx).is_ok()); // p-m (2 consonants), o-e (2 vowels)

    // Invalid cases
    assert!((rule.validate)("hello", &ctx).is_err()); // h-l-l (3 consonants), e-o (2 vowels)
    assert!((rule.validate)("world", &ctx).is_err()); // w-r-l-d (4 consonants), o (1 vowel)
    assert!((rule.validate)("house", &ctx).is_err()); // h-s (2 consonants), o-u-e (3 vowels)

    // Check error message
    let result = (rule.validate)("hello", &ctx);
    assert!(
        result
            .unwrap_err()
            .contains("must have an equal number of vowels and consonants")
    );
}

#[test]
fn test_rules_count() {
    let ctx = create_test_context();
    let rules = get_rules(&ctx);

    // Ensure we have all 17 rules
    assert_eq!(rules.len(), 17);
}

#[test]
fn test_different_contexts() {
    // Test with different min_word_length
    let ctx1 = RuleContext {
        min_word_length: 2,
        random_letter: 'x',
    };

    let ctx2 = RuleContext {
        min_word_length: 6,
        random_letter: 'z',
    };

    let rules1 = get_rules(&ctx1);
    let rules2 = get_rules(&ctx2);

    // Test min length with different contexts
    assert!((rules1[0].validate)("hi", &ctx1).is_ok());
    assert!((rules1[0].validate)("hi", &ctx2).is_err());

    // Test contains letter with different contexts
    assert!((rules1[1].validate)("example", &ctx1).is_ok());
    assert!((rules1[1].validate)("example", &ctx2).is_err());

    assert!((rules2[1].validate)("puzzle", &ctx2).is_ok());
    assert!((rules2[1].validate)("puzzle", &ctx1).is_err());
}
