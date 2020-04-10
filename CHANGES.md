# Changes

## 0.2.0

- Made Opencage and Openstreetmap responses/results serializable so it's easier to store them afterwards
  - <https://github.com/georust/geocoding/pull/31>
- Replace Failure with Thiserror
    - <https://github.com/georust/geocoding/pull/34>
- Update geo-types to 0.5
    - <https://github.com/georust/geocoding/pull/34>


## 0.1.0

- Added OpenStreetMap provider
  - <https://github.com/georust/geocoding/pull/22>
- Fixes to keep up with OpenCage schema updates
  - <https://github.com/georust/geocoding/pull/23>
- Switch to 2018 edition, use of `Failure`, more robust OpenCage schema definition, more ergonomic specification of bounds for OpenCage
  - https://github.com/georust/geocoding/pull/15
