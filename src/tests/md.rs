use crate::*;

/// test case: valid markdown, entry not done, not actually a date, date-like entry text
#[test]
fn test_valid_fake_date_not_done_dateless() {
    let entry = Entry::from_md("- [ ] (2024-06-what) this should work, kind of".to_string());
    dbg!(&entry);
    assert_eq!(
        entry.unwrap(),
        Entry {
            done: false,
            deadline: None,
            text: "(2024-06-what) this should work, kind of".to_string(),
        }
    );
}

/// test case: valid, weirdly spaced, done with date
#[test]
fn test_valid_weirdly_spaced_done_dated() {
    let entry =
        Entry::from_md("     -  [     x] (2024-06-20 20:30) weirdly spaced but ok".to_string());
    assert_eq!(
        entry.unwrap(),
        Entry {
            done: true,
            deadline: Some(
                chrono::NaiveDateTime::parse_from_str("(2024-06-20 20:30)", "(%Y-%m-%d %H:%M)")
                    .unwrap()
            ),
            text: "weirdly spaced but ok".to_string(),
        }
    );
}

// test case: invalid, undefined checkbox fill character 'a'
#[test]
fn test_invalid_bad_checkbox_fill() {
    let entry = Entry::from_md("- [a] this shouldn't work".to_string());
    dbg!(&entry);
    assert!(entry.is_err());
}
