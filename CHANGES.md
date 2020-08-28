# Changes

## 0.3.0

- Update reqwest and hyper
  - <https://github.com/georust/geocoding/pull/35>
- Upgrade geo-types
  - <https://github.com/georust/geocoding/commit/97f620688ff874f1092a6cecbe731cf15d0c3e55>
- Allow optional parameters for Opencage
  - <https://github.com/georust/geocoding/pull/38>
- Derive `Clone` for Opencage results
  - <https://github.com/georust/geocoding/pull/38/commits/61019fe0da2bb06580fcf7188eb2381a67d564d2>
- Allow usage of `rustls-tls` feature

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
