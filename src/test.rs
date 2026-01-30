use std::{thread::sleep, time::Duration};

use crate::{ExpiringMap, ExpiringSet};
#[test]
fn map_works() {
    let mut m = ExpiringMap::new(Duration::from_millis(50));
    m.insert("v", "x");
    assert_eq!(m.get(&"v"), Some(&"x"));
    sleep(Duration::from_millis(75));
    assert!(!m.contains_key(&"v"));
}

#[test]
fn set_works() {
    let mut m = ExpiringSet::new(Duration::from_millis(50));
    m.insert("v");
    assert!(m.contains_key(&"v"));
    sleep(Duration::from_millis(75));
    assert!(!m.contains_key(&"v"));
}

#[test]
fn remaining_calc() {
    let mut m = ExpiringSet::new(Duration::from_millis(50));
    m.insert("v");
    let meta = m.get_meta(&"v").unwrap();
    for _ in 0..3 {
        let a = meta.remaining();
        sleep(Duration::from_millis(1));
        let b = meta.remaining();
        assert!(a > b, "{:?} !> {:?}", a, b);
    }

    sleep(Duration::from_millis(75));
    assert!(!m.contains_key(&"v"));
}

#[test]
fn vacuum_keeps() {
    let mut m = ExpiringSet::new(Duration::from_secs(50));
    m.insert("v");
    m.vacuum();
    assert!(m.get(&"v").is_some());
}

#[test]
fn vacuum_sweeps() {
    let mut m = ExpiringSet::new(Duration::from_millis(50));
    m.insert("v");
    sleep(Duration::from_millis(75));
    m.vacuum();
    assert!(!m.inner.contains_key(&"v"));
}

#[test]
fn insert_replace() {
    let mut m = ExpiringMap::new(Duration::from_secs(5));
    m.insert("v", "x");
    assert_eq!(m.insert("v", "y").map(|v| v.value), Some("x"));
}

#[test]
fn insert_replace_sweep() {
    let mut m = ExpiringMap::new(Duration::from_millis(50));
    m.insert("v", "x");
    assert_eq!(m.insert("v", "y").map(|v| v.value), Some("x"));
    sleep(Duration::from_millis(75));
    assert!(m.insert("v", "z").is_none());
    assert_eq!(m.get("v"), Some(&"z"))
}

#[test]
fn test_borrow() {
    let mut m: ExpiringMap<String, usize> = ExpiringMap::new(Duration::from_secs(5));
    m.insert(String::from("x"), 1);
    assert_eq!(m.get("x"), Some(&1));
    assert_eq!(m.get(&String::from("x")), Some(&1));

    let mut m: ExpiringSet<String> = ExpiringSet::new(Duration::from_secs(5));
    m.insert(String::from("x"));
    assert!(m.contains("x"));
    assert!(m.contains(&String::from("x")));
}
