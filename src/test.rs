use std::{thread::sleep, time::Duration};

use crate::{ExpiringMap, ExpiringSet};
#[test]
fn map_works() {
    let mut m = ExpiringMap::new();
    m.insert("v", "x", Duration::from_millis(50));
    assert!(m.contains_key(&"v"));
    sleep(Duration::from_millis(75));
    assert!(!m.contains_key(&"v"));
}

#[test]
fn set_works() {
    let mut m = ExpiringSet::new();
    m.insert("v", Duration::from_millis(50));
    assert!(m.contains_key(&"v"));
    sleep(Duration::from_millis(75));
    assert!(!m.contains_key(&"v"));
}

#[test]
fn remaining_calc() {
    let mut m = ExpiringSet::new();
    m.insert("v", Duration::from_millis(50));
    let meta = m.get_meta(&"v").unwrap();
    dbg!(meta.remaining());
    // we allow 10ms of slop here. Should be enough;
    assert!(meta.remaining() > Duration::from_millis(40));
    assert!(meta.remaining() < Duration::from_millis(60));
    sleep(Duration::from_millis(75));
    assert!(!m.contains_key(&"v"));
}

#[test]
fn vacuum_keeps() {
    let mut m = ExpiringSet::new();
    m.insert("v", Duration::from_millis(50));
    m.vacuum();
    assert!(m.get(&"v").is_some());
}

#[test]
fn vacuum_sweeps() {
    let mut m = ExpiringSet::new();
    m.insert("v", Duration::from_millis(50));
    sleep(Duration::from_millis(75));
    m.vacuum();
    assert!(m.inner.get(&"v").is_none());
}

#[test]
fn insert_replace() {
    let mut m = ExpiringMap::new();
    m.insert("v", "x", Duration::from_secs(5));
    assert_eq!(
        m.insert("v", "y", Duration::from_secs(5)).unwrap().value,
        "x"
    );
}

#[test]
fn insert_replace_sweep() {
    let mut m = ExpiringMap::new();
    m.insert("v", "x", Duration::ZERO);
    assert!(m.insert("v", "y", Duration::from_secs(1)).is_none())
}
