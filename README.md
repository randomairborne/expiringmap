# expiringmap

A rust library implementing a TTL map.

```rust
use std::time::Duration;
use expiringmap::ExpiringMap;

fn main() {
    let mut map = ExpiringMap::new();
    map.insert("key", "value", Duration::from_millis(50));
    std::thread::sleep(Duration::from_millis(60));
    assert!(map.get(&"key").is_none());
}
```