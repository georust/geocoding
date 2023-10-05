# Changes

## unreleased

- Remove `chrono` dependency
- update geo-types to 0.7.8

### Breaking Changes

Due to previous security issues caused by the `chrono` crate
the `NaiveDateTime` was replaces by a `UnixTime` type:

```diff
- use chrono::NaiveDateTime;
- use geocoding::opencage::Timestamp;
+ use geocoding::opencage::{Timestamp, UnixTime};

  let created_http = "Mon, 16 May 2022 14:52:47 GMT".to_string();

  let ts_in_seconds = 1_652_712_767_i64;
- let created_unix = NaiveDateTime::from_timestamp(ts_in_seconds, 0);
+ let created_unix = UnixTime::from_seconds(ts_in_seconds);

  let timestamp = Timestamp { created_http, created_unix };

+ assert_eq!(ts_in_seconds, created_unix.as_seconds());
```

## 0.4.0
- Update CI to use same Rust versions as geo
- Switch GeoAdmin API to WGS84
  - <https://github.com/georust/geocoding/pull/43>
- Migrate to Github Actions
- Update tests and dependencies
- Update geo-types
- Derive Debug where necessary
- Fix OpenCage schema <https://github.com/georust/geocoding/pull/55>

## 0.3.1

- Allow usage of `rustls-tls` feature

## 0.3.0

- Update reqwest and hyper
  - <https://github.com/georust/geocoding/pull/35>
- Upgrade geo-types
  - <https://github.com/georust/geocoding/commit/97f620688ff874f1092a6cecbe731cf15d0c3e55>
- Allow optional parameters for Opencage
  - <https://github.com/georust/geocoding/pull/38>
- Derive `Clone` for Opencage results
  - <https://github.com/georust/geocoding/pull/38/commits/61019fe0da2bb06580fcf7188eb2381a67d564d2>

## 0.2.0

- Made Opencage and Openstreetmap responses/results serializable so it's easier to store them afterwards
  - <https://github.com/georust/geocoding/pull/31>
- Replace Failure with Thiserror
    - <https://github.com/georust/geocoding/pull/34>
- Update geo-types to 0.5
    - <https://github.com/georust/geocoding/pull/34>
- Update reqwest and hyper
    - <https://github.com/georust/geocoding/pull/35>


## 0.1.0

- Added OpenStreetMap provider
  - <https://github.com/georust/geocoding/pull/22>
- Fixes to keep up with OpenCage schema updates
  - <https://github.com/georust/geocoding/pull/23>
- Switch to 2018 edition, use of `Failure`, more robust OpenCage schema definition, more ergonomic specification of bounds for OpenCage
  - https://github.com/georust/geocoding/pull/15
