//! The [GeoAdmin](https://api3.geo.admin.ch) provider for geocoding in Switzerland exclusively.
//!
//! Based on the [Search API](https://api3.geo.admin.ch/services/sdiservices.html#search)
//! and [Identify Features API](https://api3.geo.admin.ch/services/sdiservices.html#identify-features)
//!
//! While GeoAdmin API is free, please respect their fair usage policy.
//!
//! ### Example
//!
//! ```
//! use geocoding::{GeoAdmin, Forward, Point};
//!
//! let geoadmin = GeoAdmin::new();
//! let address = "Seftigenstrasse 264, 3084 Wabern";
//! let res = geoadmin.forward(&address);
//! assert_eq!(res.unwrap(), vec![Point::new(7.451352119445801, 46.92793655395508)]);
//! ```
use std::fmt::Debug;

use crate::Deserialize;
use crate::GeocodingError;
use crate::InputBounds;
use crate::Point;
use crate::UA_STRING;
use crate::{Client, HeaderMap, HeaderValue, USER_AGENT};
use crate::{Forward, Reverse};
use geo_types::CoordFloat;
use num_traits::Pow;

/// An instance of the GeoAdmin geocoding service
pub struct GeoAdmin {
    client: Client,
    endpoint: String,
    sr: String,
}

/// An instance of a parameter builder for GeoAdmin geocoding
pub struct GeoAdminParams<'a, T>
where
    T: CoordFloat,
{
    searchtext: &'a str,
    origins: &'a str,
    bbox: Option<&'a InputBounds<T>>,
    limit: Option<u8>,
}

impl<'a, T> GeoAdminParams<'a, T>
where
    T: CoordFloat,
{
    /// Create a new GeoAdmin parameter builder
    /// # Example:
    ///
    /// ```
    /// use geocoding::{GeoAdmin, InputBounds, Point};
    /// use geocoding::geoadmin::{GeoAdminParams};
    ///
    /// let bbox = InputBounds::new(
    ///     (7.4513398, 46.92792859),
    ///     (7.4513662, 46.9279467),
    /// );
    /// let params = GeoAdminParams::new(&"Seftigenstrasse Bern")
    ///     .with_origins("address")
    ///     .with_bbox(&bbox)
    ///     .build();
    /// ```
    pub fn new(searchtext: &'a str) -> GeoAdminParams<'a, T> {
        GeoAdminParams {
            searchtext,
            origins: "zipcode,gg25,district,kantone,gazetteer,address,parcel",
            bbox: None,
            limit: Some(50),
        }
    }

    /// Set the `origins` property
    pub fn with_origins(&mut self, origins: &'a str) -> &mut Self {
        self.origins = origins;
        self
    }

    /// Set the `bbox` property
    pub fn with_bbox(&mut self, bbox: &'a InputBounds<T>) -> &mut Self {
        self.bbox = Some(bbox);
        self
    }

    /// Set the `limit` property
    pub fn with_limit(&mut self, limit: u8) -> &mut Self {
        self.limit = Some(limit);
        self
    }

    /// Build and return an instance of GeoAdminParams
    pub fn build(&self) -> GeoAdminParams<'a, T> {
        GeoAdminParams {
            searchtext: self.searchtext,
            origins: self.origins,
            bbox: self.bbox,
            limit: self.limit,
        }
    }
}

impl GeoAdmin {
    /// Create a new GeoAdmin geocoding instance using the default endpoint and sr
    pub fn new() -> Self {
        GeoAdmin::default()
    }

    /// Set a custom endpoint of a GeoAdmin geocoding instance
    ///
    /// Endpoint should include a trailing slash (i.e. "https://api3.geo.admin.ch/rest/services/api/")
    pub fn with_endpoint(mut self, endpoint: &str) -> Self {
        self.endpoint = endpoint.to_owned();
        self
    }

    /// Set a custom sr of a GeoAdmin geocoding instance
    ///
    /// Supported values: 21781 (LV03), 2056 (LV95), 4326 (WGS84) and 3857 (Web Pseudo-Mercator)
    pub fn with_sr(mut self, sr: &str) -> Self {
        self.sr = sr.to_owned();
        self
    }

    /// A forward-geocoding search of a location, returning a full detailed response
    ///
    /// Accepts an [`GeoAdminParams`](struct.GeoAdminParams.html) struct for specifying
    /// options, including what origins to response and whether to filter
    /// by a bounding box.
    ///
    /// Please see [the documentation](https://api3.geo.admin.ch/services/sdiservices.html#search) for details.
    ///
    /// This method passes the `format` parameter to the API.
    ///
    /// # Examples
    ///
    /// ```
    /// use geocoding::{GeoAdmin, InputBounds, Point};
    /// use geocoding::geoadmin::{GeoAdminParams, GeoAdminForwardResponse};
    ///
    /// let geoadmin = GeoAdmin::new();
    /// let bbox = InputBounds::new(
    ///     (7.4513398, 46.92792859),
    ///     (7.4513662, 46.9279467),
    /// );
    /// let params = GeoAdminParams::new(&"Seftigenstrasse Bern")
    ///     .with_origins("address")
    ///     .with_bbox(&bbox)
    ///     .build();
    /// let res: GeoAdminForwardResponse<f64> = geoadmin.forward_full(&params).unwrap();
    /// let result = &res.features[0];
    /// assert_eq!(
    ///     result.properties.label,
    ///     "Seftigenstrasse 264 <b>3084 Wabern</b>",
    /// );
    /// ```
    pub fn forward_full<T>(
        &self,
        params: &GeoAdminParams<T>,
    ) -> Result<GeoAdminForwardResponse<T>, GeocodingError>
    where
        T: CoordFloat,
        for<'de> T: Deserialize<'de>,
    {
        // For lifetime issues
        let bbox;
        let limit;

        let mut query = vec![
            ("searchText", params.searchtext),
            ("type", "locations"),
            ("origins", params.origins),
            ("sr", &self.sr),
            ("geometryFormat", "geojson"),
        ];

        if let Some(bb) = params.bbox.cloned().as_mut() {
            if vec!["4326", "3857"].contains(&self.sr.as_str()) {
                *bb = InputBounds::new(
                    wgs84_to_lv03(&bb.minimum_lonlat),
                    wgs84_to_lv03(&bb.maximum_lonlat),
                );
            }
            bbox = String::from(*bb);
            query.push(("bbox", &bbox));
        }

        if let Some(lim) = params.limit {
            limit = lim.to_string();
            query.push(("limit", &limit));
        }

        let resp = self
            .client
            .get(&format!("{}SearchServer", self.endpoint))
            .query(&query)
            .send()?
            .error_for_status()?;
        let res: GeoAdminForwardResponse<T> = resp.json()?;
        Ok(res)
    }
}

impl Default for GeoAdmin {
    fn default() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(UA_STRING));
        let client = Client::builder()
            .default_headers(headers)
            .build()
            .expect("Couldn't build a client!");
        GeoAdmin {
            client,
            endpoint: "https://api3.geo.admin.ch/rest/services/api/".to_string(),
            sr: "4326".to_string(),
        }
    }
}

impl<T> Forward<T> for GeoAdmin
where
    T: CoordFloat,
    for<'de> T: Deserialize<'de>,
{
    /// A forward-geocoding lookup of an address. Please see [the documentation](https://api3.geo.admin.ch/services/sdiservices.html#search) for details.
    ///
    /// This method passes the `type`,  `origins`, `limit` and `sr` parameter to the API.
    fn forward(&self, place: &str) -> Result<Vec<Point<T>>, GeocodingError> {
        let resp = self
            .client
            .get(&format!("{}SearchServer", self.endpoint))
            .query(&[
                ("searchText", place),
                ("type", "locations"),
                ("origins", "address"),
                ("limit", "1"),
                ("sr", &self.sr),
                ("geometryFormat", "geojson"),
            ])
            .send()?
            .error_for_status()?;
        let res: GeoAdminForwardResponse<T> = resp.json()?;
        // return easting & northing consistent
        let results = if vec!["2056", "21781"].contains(&self.sr.as_str()) {
            res.features
                .iter()
                .map(|feature| Point::new(feature.properties.y, feature.properties.x)) // y = west-east, x = north-south
                .collect()
        } else {
            res.features
                .iter()
                .map(|feature| Point::new(feature.properties.x, feature.properties.y)) // x = west-east, y = north-south
                .collect()
        };
        Ok(results)
    }
}

impl<T> Reverse<T> for GeoAdmin
where
    T: CoordFloat,
    for<'de> T: Deserialize<'de>,
{
    /// A reverse lookup of a point. More detail on the format of the
    /// returned `String` can be found [here](https://api3.geo.admin.ch/services/sdiservices.html#identify-features)
    ///
    /// This method passes the `format` parameter to the API.
    fn reverse(&self, point: &Point<T>) -> Result<Option<String>, GeocodingError> {
        let resp = self
            .client
            .get(&format!("{}MapServer/identify", self.endpoint))
            .query(&[
                (
                    "geometry",
                    format!(
                        "{},{}",
                        point.x().to_f64().unwrap(),
                        point.y().to_f64().unwrap()
                    )
                    .as_str(),
                ),
                ("geometryType", "esriGeometryPoint"),
                ("layers", "all:ch.bfs.gebaeude_wohnungs_register"),
                ("mapExtent", "0,0,100,100"),
                ("imageDisplay", "100,100,100"),
                ("tolerance", "50"),
                ("geometryFormat", "geojson"),
                ("sr", &self.sr),
                ("lang", "en"),
            ])
            .send()?
            .error_for_status()?;
        let res: GeoAdminReverseResponse = resp.json()?;
        if !res.results.is_empty() {
            let properties = &res.results[0].properties;
            let address = format!(
                "{}, {} {}",
                properties.strname_deinr, properties.dplz4, properties.dplzname
            );
            Ok(Some(address))
        } else {
            Ok(None)
        }
    }
}

// Approximately transform Point from WGS84 to LV03
//
// See [the documentation](https://www.swisstopo.admin.ch/content/swisstopo-internet/en/online/calculation-services/_jcr_content/contentPar/tabs/items/documents_publicatio/tabPar/downloadlist/downloadItems/19_1467104393233.download/ch1903wgs84_e.pdf) for more details
fn wgs84_to_lv03<T>(p: &Point<T>) -> Point<T>
where
    T: CoordFloat,
{
    let lambda = (p.x().to_f64().unwrap() * 3600.0 - 26782.5) / 10000.0;
    let phi = (p.y().to_f64().unwrap() * 3600.0 - 169028.66) / 10000.0;
    let x = 2600072.37 + 211455.93 * lambda
        - 10938.51 * lambda * phi
        - 0.36 * lambda * phi.pow(2)
        - 44.54 * lambda.pow(3);
    let y = 1200147.07 + 308807.95 * phi + 3745.25 * lambda.pow(2) + 76.63 * phi.pow(2)
        - 194.56 * lambda.pow(2) * phi
        + 119.79 * phi.pow(3);
    Point::new(
        T::from(x - 2000000.0).unwrap(),
        T::from(y - 1000000.0).unwrap(),
    )
}
/// The top-level full JSON (GeoJSON Feature Collection) response returned by a forward-geocoding request
///
/// See [the documentation](https://api3.geo.admin.ch/services/sdiservices.html#search) for more details
///
///```json
///{
///     "type": "FeatureCollection",
///     "features": [
///         {
///             "properties": {
///                 "origin": "address",
///                 "geom_quadindex": "021300220302203002031",
///                 "weight": 1512,
///                 "zoomlevel": 10,
///                 "lon": 7.451352119445801,
///                 "detail": "seftigenstrasse 264 3084 wabern 355 koeniz ch be",
///                 "rank": 7,
///                 "lat": 46.92793655395508,
///                 "num": 264,
///                 "y": 2600968.75,
///                 "x": 1197427.0,
///                 "label": "Seftigenstrasse 264 <b>3084 Wabern</b>"
///                 "id": 1420809,
///             }
///         }
///     ]
/// }
///```
#[derive(Debug, Deserialize)]
pub struct GeoAdminForwardResponse<T>
where
    T: CoordFloat,
{
    pub features: Vec<GeoAdminForwardLocation<T>>,
}

/// A forward geocoding location
#[derive(Debug, Deserialize)]
pub struct GeoAdminForwardLocation<T>
where
    T: CoordFloat,
{
    id: Option<usize>,
    pub properties: ForwardLocationProperties<T>,
}

/// Forward Geocoding location attributes
#[derive(Clone, Debug, Deserialize)]
pub struct ForwardLocationProperties<T> {
    pub origin: String,
    pub geom_quadindex: String,
    pub weight: u32,
    pub rank: u32,
    pub detail: String,
    pub lat: T,
    pub lon: T,
    pub num: Option<usize>,
    pub x: T,
    pub y: T,
    pub label: String,
    pub zoomlevel: u32,
}

/// The top-level full JSON (GeoJSON FeatureCollection) response returned by a reverse-geocoding request
///
/// See [the documentation](https://api3.geo.admin.ch/services/sdiservices.html#identify-features) for more details
///
///```json
/// {
///     "results": [
///         {
///             "type": "Feature"
///             "id": "1272199_0"
///             "attributes": {
///                 "xxx": "xxx",
///                 "...": "...",
///             },
///             "layerBodId": "ch.bfs.gebaeude_wohnungs_register",
///             "layerName": "Register of Buildings and Dwellings",
///         }
///     ]
/// }
///```
#[derive(Debug, Deserialize)]
pub struct GeoAdminReverseResponse {
    pub results: Vec<GeoAdminReverseLocation>,
}

/// A reverse geocoding result
#[derive(Debug, Deserialize)]
pub struct GeoAdminReverseLocation {
    id: String,
    #[serde(rename = "featureId")]
    pub feature_id: String,
    #[serde(rename = "layerBodId")]
    pub layer_bod_id: String,
    #[serde(rename = "layerName")]
    pub layer_name: String,
    pub properties: ReverseLocationAttributes,
}

/// Reverse geocoding result attributes
#[derive(Clone, Debug, Deserialize)]
pub struct ReverseLocationAttributes {
    pub egid: Option<String>,
    pub ggdenr: u32,
    pub ggdename: String,
    pub gdekt: String,
    pub edid: Option<String>,
    pub egaid: u32,
    pub deinr: Option<String>,
    pub dplz4: u32,
    pub dplzname: String,
    pub egrid: Option<String>,
    pub esid: u32,
    pub strname: Vec<String>,
    pub strsp: Vec<String>,
    pub strname_deinr: String,
    pub label: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_with_sr_forward_test() {
        let geoadmin = GeoAdmin::new().with_sr("2056");
        let address = "Seftigenstrasse 264, 3084 Wabern";
        let res = geoadmin.forward(address);
        assert_eq!(res.unwrap(), vec![Point::new(2_600_968.75, 1_197_427.0)]);
    }

    #[test]
    fn new_with_endpoint_forward_test() {
        let geoadmin =
            GeoAdmin::new().with_endpoint("https://api3.geo.admin.ch/rest/services/api/");
        let address = "Seftigenstrasse 264, 3084 Wabern";
        let res = geoadmin.forward(address);
        assert_eq!(
            res.unwrap(),
            vec![Point::new(7.451352119445801, 46.92793655395508)]
        );
    }

    #[test]
    fn with_sr_forward_full_test() {
        let geoadmin = GeoAdmin::new().with_sr("2056");
        let bbox = InputBounds::new((2_600_967.75, 1_197_426.0), (2_600_969.75, 1_197_428.0));
        let params = GeoAdminParams::new("Seftigenstrasse Bern")
            .with_origins("address")
            .with_bbox(&bbox)
            .build();
        let res: GeoAdminForwardResponse<f64> = geoadmin.forward_full(&params).unwrap();
        let result = &res.features[0];
        assert_eq!(
            result.properties.label,
            "Seftigenstrasse 264 <b>3084 Wabern</b>",
        );
    }

    #[test]
    fn forward_full_test() {
        let geoadmin = GeoAdmin::new();
        let bbox = InputBounds::new((7.4513398, 46.92792859), (7.4513662, 46.9279467));
        let params = GeoAdminParams::new("Seftigenstrasse Bern")
            .with_origins("address")
            .with_bbox(&bbox)
            .build();
        let res: GeoAdminForwardResponse<f64> = geoadmin.forward_full(&params).unwrap();
        let result = &res.features[0];
        assert_eq!(
            result.properties.label,
            "Seftigenstrasse 264 <b>3084 Wabern</b>",
        );
    }

    #[test]
    fn forward_test() {
        let geoadmin = GeoAdmin::new();
        let address = "Seftigenstrasse 264, 3084 Wabern";
        let res = geoadmin.forward(address);
        assert_eq!(
            res.unwrap(),
            vec![Point::new(7.451352119445801, 46.92793655395508)]
        );
    }

    #[test]
    fn with_sr_reverse_test() {
        let geoadmin = GeoAdmin::new().with_sr("2056");
        let p = Point::new(2_600_968.75, 1_197_427.0);
        let res = geoadmin.reverse(&p);
        assert_eq!(
            res.unwrap(),
            Some("Seftigenstrasse 264, 3084 Wabern".to_string()),
        );
    }

    #[test]
    fn reverse_test() {
        let geoadmin = GeoAdmin::new();
        let p = Point::new(7.451352119445801, 46.92793655395508);
        let res = geoadmin.reverse(&p);
        assert_eq!(
            res.unwrap(),
            Some("Seftigenstrasse 264, 3084 Wabern".to_string()),
        );
    }
}
