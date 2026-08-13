use minrx::{BuildError, Match, MatchOptions, Regex, RegexBuilder};

#[test]
fn test_simple_literal_match() {
    let mut re = Regex::new("abc").unwrap();
    assert!(re.is_match("xxabcxx").unwrap());
}

#[test]
fn test_simple_literal_no_match() {
    let mut re = Regex::new("abc").unwrap();
    assert!(!re.is_match("xxabxx").unwrap());
}

#[test]
fn test_case_sensitive_default() {
    let mut re = Regex::new("abc").unwrap();
    assert!(!re.is_match("ABC").unwrap());
}

#[test]
fn test_find_matches_whole_match() {
    let mut re = Regex::new("a+b").unwrap();
    let expected = Some(Match { start: 2, end: 6 });
    let matches = re.find_matches("xxaaab").unwrap().unwrap();
    assert_eq!(matches[0], expected);
}

#[test]
fn test_find_matches_no_match_returns_none() {
    let mut re = Regex::new("xyz").unwrap();
    let result = re.find_matches("abc").unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_is_match_no_match() {
    let mut re = Regex::new("xyz").unwrap();
    assert!(!re.is_match("abc").unwrap());
}

#[test]
fn test_capture_count_no_groups() {
    let re = Regex::new("abc").unwrap();
    assert_eq!(re.capture_count(), 0);
}

#[test]
fn test_capture_count_multiple_groups() {
    let re = Regex::new("(a+)(b+)(c+)").unwrap();
    assert_eq!(re.capture_count(), 3);
}

#[test]
fn test_capture_groups_positions() {
    let mut re = Regex::new("(a+)(b+)").unwrap();
    let matches = re.find_matches("aaabbb").unwrap().unwrap();
    assert_eq!(matches.len(), 3);
    assert_eq!(matches[0], Some(Match { start: 0, end: 6 }));
    assert_eq!(matches[1], Some(Match { start: 0, end: 3 }));
    assert_eq!(matches[2], Some(Match { start: 3, end: 6 }));
}

#[test]
fn test_nested_groups() {
    let mut re = Regex::new("((a)(b))").unwrap();
    assert_eq!(re.capture_count(), 3);
    let matches = re.find_matches("ab").unwrap().unwrap();
    assert_eq!(matches[0], Some(Match { start: 0, end: 2 }));
    assert_eq!(matches[1], Some(Match { start: 0, end: 2 }));
    assert_eq!(matches[2], Some(Match { start: 0, end: 1 }));
    assert_eq!(matches[3], Some(Match { start: 1, end: 2 }));
}

#[test]
fn test_non_participating_group_is_none() {
    let mut re = Regex::new("(a)|(b)").unwrap();
    let matches = re.find_matches("b").unwrap().unwrap();
    assert_eq!(matches[0], Some(Match { start: 0, end: 1 }));
    assert_eq!(matches[1], None);
    assert_eq!(matches[2], Some(Match { start: 0, end: 1 }));
}

#[test]
fn test_alternation() {
    let mut re = Regex::new("cat|dog").unwrap();
    assert!(re.is_match("I have a dog").unwrap());
    assert!(re.is_match("I have a cat").unwrap());
    assert!(!re.is_match("I have a bird").unwrap());
}

#[test]
fn test_character_class_range() {
    let mut re = Regex::new("[0-9]+").unwrap();
    let expected = Some(Match { start: 3, end: 6 });
    let matches = re.find_matches("abc123def").unwrap().unwrap();
    assert_eq!(matches[0], expected);
}

#[test]
fn test_negated_character_class() {
    let mut re = Regex::new("[^0-9]+").unwrap();
    let expected = Some(Match { start: 3, end: 6 });
    let matches = re.find_matches("123abc456").unwrap().unwrap();
    assert_eq!(matches[0], expected);
}

#[test]
fn test_posix_class_digit() {
    let mut re = Regex::new("[[:digit:]]+").unwrap();
    assert!(re.is_match("42").unwrap());
}

#[test]
fn test_posix_class_alpha() {
    let mut re = Regex::new("[[:alpha:]]+").unwrap();
    assert!(re.is_match("hello").unwrap());
    assert!(!re.is_match("123").unwrap());
}

#[test]
fn test_interval_exact_repetition() {
    let mut re = Regex::new("a{3}").unwrap();
    let expected = Some(Match { start: 0, end: 3 });
    let matches = re.find_matches("aaaa").unwrap().unwrap();
    assert_eq!(matches[0], expected);
}

#[test]
fn test_interval_min_max_repetition() {
    let mut re = Regex::new("a{2,4}").unwrap();
    let expected = Some(Match { start: 0, end: 4 });
    let matches = re.find_matches("aaaaa").unwrap().unwrap();
    assert_eq!(matches[0], expected);
}

#[test]
fn test_interval_min_only_repetition() {
    let mut re = Regex::new("a{2,}").unwrap();
    let expected = Some(Match { start: 0, end: 5 });
    let matches = re.find_matches("aaaaa").unwrap().unwrap();
    assert_eq!(matches[0], expected);
}

#[test]
fn test_optional_operator() {
    let mut re = Regex::new("colou?r").unwrap();
    assert!(re.is_match("color").unwrap());
    assert!(re.is_match("colour").unwrap());
}

#[test]
fn test_star_operator_zero_occurrences() {
    let mut re = Regex::new("ab*c").unwrap();
    assert!(re.is_match("ac").unwrap());
}

#[test]
fn test_plus_operator_requires_one() {
    let mut re = Regex::new("ab+c").unwrap();
    assert!(!re.is_match("ac").unwrap());
    assert!(re.is_match("abc").unwrap());
}

#[test]
fn test_anchors_start_and_end() {
    let mut re = Regex::new("^hello$").unwrap();
    assert!(re.is_match("hello").unwrap());
    assert!(!re.is_match("hello world").unwrap());
}

#[test]
fn test_empty_pattern_matches_empty_string() {
    let mut re = Regex::new("").unwrap();
    let expected = Some(Match { start: 0, end: 0 });
    let matches = re.find_matches("anything").unwrap().unwrap();
    assert_eq!(matches[0], expected);
}

#[test]
fn test_empty_haystack_no_match() {
    let mut re = Regex::new("a+").unwrap();
    let result = re.find_matches("").unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_bytes_pattern_and_haystack() {
    let mut re = Regex::new(b"a+b".as_slice()).unwrap();
    assert!(re.is_match(b"aaab".as_slice()).unwrap());
}

#[test]
fn test_match_equality() {
    let a = Match { start: 1, end: 2 };
    let b = Match { start: 1, end: 2 };
    assert_eq!(a, b);
}

#[test]
fn test_match_inequality() {
    let a = Match { start: 1, end: 2 };
    let b = Match { start: 1, end: 3 };
    assert_ne!(a, b);
}

#[test]
fn test_match_ordering() {
    let a = Match { start: 1, end: 2 };
    let b = Match { start: 1, end: 3 };
    assert!(a < b);
}

#[test]
fn test_case_insensitive_option() {
    let mut re = RegexBuilder::new()
        .case_insensitive(true)
        .build("hello")
        .unwrap();
    assert!(re.is_match("HELLO").unwrap());
    assert!(re.is_match("HeLLo").unwrap());
}

#[test]
fn test_case_insensitive_disabled_by_default() {
    let mut re = RegexBuilder::new().build("hello").unwrap();
    assert!(!re.is_match("HELLO").unwrap());
}

#[test]
fn test_swap_greed_makes_plus_minimal() {
    let mut re = RegexBuilder::new().swap_greed(true).build("a+").unwrap();
    let expected = Some(Match { start: 0, end: 1 });
    let matches = re.find_matches("aaa").unwrap().unwrap();
    assert_eq!(matches[0], expected);
}

#[test]
fn test_default_greed_is_maximal() {
    let mut re = Regex::new("a+").unwrap();
    let expected = Some(Match { start: 0, end: 3 });
    let matches = re.find_matches("aaa").unwrap().unwrap();
    assert_eq!(matches[0], expected);
}

#[test]
fn test_multi_line_caret_matches_after_newline() {
    let mut re = RegexBuilder::new().multi_line(true).build("^b").unwrap();
    let expected = Some(Match { start: 2, end: 3 });
    let matches = re.find_matches("a\nb").unwrap().unwrap();
    assert_eq!(matches[0], expected);
}

#[test]
fn test_multi_line_dollar_matches_before_newline() {
    let mut re = RegexBuilder::new().multi_line(true).build("a$").unwrap();
    let expected = Some(Match { start: 0, end: 1 });
    let matches = re.find_matches("a\nb").unwrap().unwrap();
    assert_eq!(matches[0], expected);
}

#[test]
fn test_multi_line_dot_excludes_newline() {
    let mut re = RegexBuilder::new().multi_line(true).build("a.b").unwrap();
    let result = re.find_matches("a\nb").unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_no_substrings_is_match_still_works() {
    let mut re = RegexBuilder::new()
        .no_substrings(true)
        .build("a+b")
        .unwrap();
    assert!(re.is_match("aaab").unwrap());
    assert!(!re.is_match("ccc").unwrap());
}

#[test]
fn test_gnu_extensions_word_boundary() {
    let mut re = RegexBuilder::new()
        .gnu_extensions(true)
        .build(r"\bfoo\b")
        .unwrap();
    assert!(re.is_match("a foo b").unwrap());
    assert!(!re.is_match("afoob").unwrap());
}

#[test]
fn test_bsd_extensions_word_boundary() {
    let mut re = RegexBuilder::new()
        .bsd_extensions(true)
        .build(r"\<foo\>")
        .unwrap();
    assert!(re.is_match("a foo b").unwrap());
    assert!(!re.is_match("afoob").unwrap());
}

#[test]
fn test_brace_compat_literal_brace() {
    let mut re = RegexBuilder::new().brace_compat(true).build("a{b").unwrap();
    assert!(re.is_match("a{b").unwrap());
}

#[test]
fn test_escapes_in_brackets() {
    let mut re = RegexBuilder::new()
        .escapes_in_brackets(true)
        .build(r"[\]]")
        .unwrap();
    assert!(re.is_match("]").unwrap());
}

#[test]
fn test_extended_no_op() {
    let mut builder = RegexBuilder::new();
    builder.extended(false);
    let mut re = builder.build("a+b").unwrap();
    assert!(re.is_match("aaab").unwrap());
}

#[test]
fn test_builder_default_matches_new() {
    let mut re = RegexBuilder::default().build("abc").unwrap();
    assert!(re.is_match("abc").unwrap());
}

#[test]
fn test_builder_reusable_across_multiple_builds() {
    let mut builder = RegexBuilder::new();
    builder.case_insensitive(true);
    let mut re1 = builder.build("abc").unwrap();
    let mut re2 = builder.build("xyz").unwrap();
    assert!(re1.is_match("ABC").unwrap());
    assert!(re2.is_match("XYZ").unwrap());
}

#[test]
fn test_match_options_default_matches_new_behavior() {
    let mut re = Regex::new("abc").unwrap();
    let opts = MatchOptions::default();
    assert!(re.is_match_with("abc", opts).unwrap());
}

#[test]
fn test_not_bol_disables_caret_at_start() {
    let mut re = Regex::new("^abc").unwrap();
    let mut opts = MatchOptions::new();
    opts.not_bol(true);
    let result = re.find_matches_with("abc", opts).unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_not_bol_false_allows_caret_at_start() {
    let mut re = Regex::new("^abc").unwrap();
    let mut opts = MatchOptions::new();
    opts.not_bol(false);
    let expected = Some(Match { start: 0, end: 3 });
    let matches = re.find_matches_with("abc", opts).unwrap().unwrap();
    assert_eq!(matches[0], expected);
}

#[test]
fn test_not_eol_disables_dollar_at_end() {
    let mut re = Regex::new("abc$").unwrap();
    let mut opts = MatchOptions::new();
    opts.not_eol(true);
    let result = re.find_matches_with("abc", opts).unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_first_subexpr_captures_first_occurrence() {
    let mut re = Regex::new("(a|b)+").unwrap();
    let mut opts = MatchOptions::new();
    opts.first_subexpr(true);
    let expected = Some(Match { start: 0, end: 1 });
    let matches = re.find_matches_with("ab", opts).unwrap().unwrap();
    assert_eq!(matches[1], expected);
}

#[test]
fn test_default_captures_last_occurrence() {
    let mut re = Regex::new("(a|b)+").unwrap();
    let expected = Some(Match { start: 1, end: 2 });
    let matches = re.find_matches("ab").unwrap().unwrap();
    assert_eq!(matches[1], expected);
}

#[test]
fn test_find_iter_multiple_matches() {
    let mut re = Regex::new("a+").unwrap();
    let expected = vec![Match { start: 0, end: 2 }, Match { start: 6, end: 9 }];
    let results: Vec<Match> = re.find_iter("aa bb aaa").map(|m| m.unwrap()).collect();
    assert_eq!(results, expected);
}

#[test]
fn test_find_iter_no_matches() {
    let mut re = Regex::new("xyz").unwrap();
    let results: Vec<_> = re.find_iter("abc").collect();
    assert!(results.is_empty());
}

#[test]
fn test_find_iter_handles_empty_matches_by_advancing() {
    let mut re = Regex::new("a*").unwrap();
    let expected = vec![
        Match { start: 0, end: 0 },
        Match { start: 1, end: 3 },
        Match { start: 3, end: 3 },
    ];
    let results: Vec<Match> = re.find_iter("baa").map(|m| m.unwrap()).collect();
    assert_eq!(results, expected);
}

#[test]
fn test_find_iter_single_full_match() {
    let mut re = Regex::new("^abc$").unwrap();
    let expected = vec![Match { start: 0, end: 3 }];
    let results: Vec<Match> = re.find_iter("abc").map(|m| m.unwrap()).collect();
    assert_eq!(results, expected);
}

#[test]
fn test_find_iter_with_flags_not_bol() {
    let mut re = Regex::new("^a").unwrap();
    let mut opts = MatchOptions::new();
    opts.not_bol(true);
    let results: Vec<_> = re.find_iter_with_flags("aaa", opts).collect();
    assert!(results.is_empty());
}

#[test]
fn test_error_unbalanced_paren() {
    let result = Regex::new("(a");
    assert!(matches!(result, Err(BuildError::UnbalancedParen(_))));
}

#[test]
fn test_error_unbalanced_bracket() {
    let result = Regex::new("[a");
    assert!(matches!(result, Err(BuildError::UnbalancedBracket(_))));
}

#[test]
fn test_error_bad_repetition_leading_star() {
    let result = Regex::new("*abc");
    assert!(matches!(result, Err(BuildError::BadRepetition(_))));
}

#[test]
fn test_error_message_is_not_empty() {
    let result = Regex::new("(a");
    match result {
        Err(BuildError::UnbalancedParen(msg)) => assert!(!msg.is_empty()),
        _ => panic!(),
    }
}

#[test]
fn test_valid_pattern_does_not_error() {
    let result = Regex::new("a(b|c)+d?");
    assert!(result.is_ok());
}

#[test]
fn test_regex_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<Regex>();
}

#[test]
fn test_repeated_is_match_calls_are_stable() {
    let mut re = Regex::new("a+b").unwrap();
    assert!(re.is_match("aaab").unwrap());
    assert!(re.is_match("aaab").unwrap());
    assert!(!re.is_match("ccc").unwrap());
    assert!(re.is_match("ab").unwrap());
}

#[test]
fn test_repeated_find_matches_calls_are_stable() {
    let mut re = Regex::new("[0-9]+").unwrap();
    let first = re.find_matches("a1b").unwrap().unwrap();
    let second = re.find_matches("cc22dd").unwrap().unwrap();
    assert_eq!(first[0], Some(Match { start: 1, end: 2 }));
    assert_eq!(second[0], Some(Match { start: 2, end: 4 }));
}

#[test]
fn test_leftmost_longest_match_semantics() {
    let mut re = Regex::new("a|ab|abc").unwrap();
    let expected = Some(Match { start: 0, end: 3 });
    let matches = re.find_matches("abcd").unwrap().unwrap();
    assert_eq!(matches[0], expected);
}

#[test]
fn test_leftmost_match_preferred_over_later_longer() {
    let mut re = Regex::new("a+").unwrap();
    let expected = Some(Match { start: 0, end: 1 });
    let matches = re.find_matches("a bb aaaaa").unwrap().unwrap();
    assert_eq!(matches[0], expected);
}

#[test]
fn test_dot_matches_any_single_character() {
    let mut re = Regex::new("a.c").unwrap();
    assert!(re.is_match("abc").unwrap());
    assert!(re.is_match("axc").unwrap());
    assert!(!re.is_match("ac").unwrap());
}

#[test]
fn test_backslash_escapes_metacharacter() {
    let mut re = Regex::new(r"a\.b").unwrap();
    assert!(re.is_match("a.b").unwrap());
    assert!(!re.is_match("axb").unwrap());
}

#[test]
fn test_multiple_capture_groups_with_alternation() {
    let mut re = Regex::new("(foo)|(bar)|(baz)").unwrap();
    let matches = re.find_matches("baz").unwrap().unwrap();
    assert_eq!(matches[0], Some(Match { start: 0, end: 3 }));
    assert_eq!(matches[1], None);
    assert_eq!(matches[2], None);
    assert_eq!(matches[3], Some(Match { start: 0, end: 3 }));
}
