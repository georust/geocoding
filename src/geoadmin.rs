//! The [GeoAdmin] (https://api3.geo.admin.ch) provider for geocoding in Switzerland exclusively.
//!
//! Based on the [Search API] (https://api3.geo.admin.ch/services/sdiservices.html#search)
//! and [Identify Features API] (https://api3.geo.admin.ch/services/sdiservices.html#identify-features)
//!
//! It uses the local swiss coordinate reference system [CH1903+ / LV95]
//! (https://www.swisstopo.admin.ch/en/knowledge-facts/surveying-geodesy/reference-frames/local/lv95.html)
//! (EPSG:2056) for input and output coordinates. Be aware of the switched axis names!
//!
//! ### Example
//!
//! ```
//! use geocoding::{GeoAdmin, Forward, Point};
//!
//! let geoadmin = GeoAdmin::new();
//! let address = "Seftigenstrasse 264, 3084 Wabern";
//! let res = geoadmin.forward(&address);
//! assert_eq!(res.unwrap(), vec![Point::new(2_600_968.75, 1_197_427.0)]);
//! ```
use crate::Deserialize;
use crate::GeocodingError;
use crate::InputBounds;
use crate::Point;
use crate::UA_STRING;
use crate::{Client, HeaderMap, HeaderValue, USER_AGENT};
use crate::{Forward, Reverse};
use num_traits::Float;

/// An instance of the GeoAdmin geocoding service
pub struct GeoAdmin {
    client: Client,
    endpoint: String,
}

/// An instance of a parameter builder for GeoAdmin geocoding
pub struct GeoAdminParams<'a, T>
where
    T: Float,
{
    searchtext: &'a str,
    origins: &'a str,
    bbox: Option<&'a InputBounds<T>>,
    limit: Option<u8>,
}

impl<'a, T> GeoAdminParams<'a, T>
where
    T: Float,
{
    /// Create a new GeoAdmin parameter builder
    /// # Example:
    ///
    /// ```
    /// use geocoding::{GeoAdmin, InputBounds, Point};
    /// use geocoding::geoadmin::{GeoAdminParams};
    ///
    /// let bbox = InputBounds::new(
    ///     (2600967.75, 1197426.0),
    ///     (2600969.75, 1197428.0),
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
    /// Create a new GeoAdmin geocoding instance using the default endpoint
    pub fn new() -> Self {
        GeoAdmin::new_with_endpoint("https://api3.geo.admin.ch/rest/services/api/".to_string())
    }

    /// Create a new GeoAdmin geocoding instance with a custom endpoint.
    ///
    /// Endpoint should include a trailing slash (i.e. "https://api3.geo.admin.ch/rest/services/api/")
    pub fn new_with_endpoint(endpoint: String) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(UA_STRING));
        let client = Client::builder()
            .default_headers(headers)
            .build()
            .expect("Couldn't build a client!");
        GeoAdmin { client, endpoint }
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
    ///     (2600967.75, 1197426.0),
    ///     (2600969.75, 1197428.0),
    /// );
    /// let params = GeoAdminParams::new(&"Seftigenstrasse Bern")
    ///     .with_origins("address")
    ///     .with_bbox(&bbox)
    ///     .build();
    /// let res: GeoAdminForwardResponse<f64> = geoadmin.forward_full(&params).unwrap();
    /// let result = &res.results[0];
    /// assert_eq!(
    ///     result.attrs.label,
    ///     "Seftigenstrasse 264 <b>3084 Wabern</b>",
    /// );
    /// ```
    pub fn forward_full<T>(
        &self,
        params: &GeoAdminParams<T>,
    ) -> Result<GeoAdminForwardResponse<T>, GeocodingError>
    where
        T: Float,
        for<'de> T: Deserialize<'de>,
    {
        let searchtype = String::from("locations");
        let sr = String::from("2056");

        // For lifetime issues
        let bbox;
        let limit;

        let mut query = vec![
            (&"searchText", params.searchtext),
            (&"type", &searchtype),
            (&"origins", params.origins),
            (&"sr", &sr),
        ];

        if let Some(bb) = params.bbox {
            bbox = String::from(*bb);
            query.push((&"bbox", &bbox));
        }

        if let Some(lim) = params.limit {
            limit = lim.to_string();
            query.push((&"limit", &limit));
        }

        let mut resp = self
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
        Self::new()
    }
}

impl<T> Forward<T> for GeoAdmin
where
    T: Float,
    for<'de> T: Deserialize<'de>,
{
    /// A forward-geocoding lookup of an address. Please see [the documentation](https://api3.geo.admin.ch/services/sdiservices.html#search) for details.
    ///
    /// This method passes the `type`,  `origins`, `limit` and `sr` parameter to the API.
    fn forward(&self, place: &str) -> Result<Vec<Point<T>>, GeocodingError> {
        let mut resp = self
            .client
            .get(&format!("{}SearchServer", self.endpoint))
            .query(&[
                (&"searchText", place),
                (&"type", &String::from("locations")),
                (&"origins", &String::from("address")),
                (&"limit", &String::from("1")),
                (&"sr", &String::from("2056")),
            ])
            .send()?
            .error_for_status()?;
        let res: GeoAdminForwardResponse<T> = resp.json()?;
        Ok(res
            .results
            .iter()
            .map(|res| Point::new(res.attrs.y, res.attrs.x)) // y = west-east, x = north-south
            .collect())
    }
}

impl<T> Reverse<T> for GeoAdmin
where
    T: Float,
    for<'de> T: Deserialize<'de>,
{
    /// A reverse lookup of a point. More detail on the format of the
    /// returned `String` can be found [here](https://api3.geo.admin.ch/services/sdiservices.html#identify-features)
    ///
    /// This method passes the `format` parameter to the API.
    fn reverse(&self, point: &Point<T>) -> Result<Option<String>, GeocodingError> {
        let mut resp = self
            .client
            .get(&format!("{}MapServer/identify", self.endpoint))
            .query(&[
                (
                    &"geometry",
                    &format!(
                        "{},{}",
                        point.x().to_f64().unwrap(),
                        point.y().to_f64().unwrap()
                    ),
                ),
                (&"geometryType", &String::from("esriGeometryPoint")),
                (
                    &"layers",
                    &String::from("all:ch.bfs.gebaeude_wohnungs_register"),
                ),
                (&"mapExtent", &String::from("0,0,100,100")),
                (&"imageDisplay", &String::from("100,100,100")),
                (&"tolerance", &String::from("50")),
                (&"geometryFormat", &String::from("geojson")),
                (&"sr", &String::from("2056")),
                (&"lang", &String::from("en")),
            ])
            .send()?
            .error_for_status()?;
        let res: GeoAdminReverseResponse = resp.json()?;
        if !res.results.is_empty() {
            let properties = &res.results[0].properties;
            let address = format!(
                "{} {}, {} {}",
                properties.strname1, properties.deinr, properties.plz4, properties.plzname
            );
            Ok(Some(address))
        } else {
            Ok(None)
        }
    }
}
/// The top-level full JSON response returned by a forward-geocoding request
///
/// See [the documentation](https://api3.geo.admin.ch/services/sdiservices.html#search) for more details
///
///```json
///{
///     "results": [
///         {
///             "id": 1420809,
///             "weight": 1512,
///             "attrs": {
///                 "origin": "address",
///                 "geom_quadindex": "021300220302203002031",
///                 "@geodist": 8.957613945007324,
///                 "zoomlevel": 10,
///                 "featureId": "1272199_0",
///                 "lon": 7.451352119445801,
///                 "detail": "seftigenstrasse 264 3084 wabern 355 koeniz ch be",
///                 "rank": 7,
///                 "geom_st_box2d": "BOX(2600968.668 1197426.954,2600968.668 1197426.954)",
///                 "lat": 46.92793655395508,
///                 "num": 264,
///                 "y": 2600968.75,
///                 "x": 1197427.0,
///                 "label": "Seftigenstrasse 264 <b>3084 Wabern</b>"
///             }
///         }
///     ]
/// }
///```
#[derive(Debug, Deserialize)]
pub struct GeoAdminForwardResponse<T>
where
    T: Float,
{
    pub results: Vec<GeoAdminForwardLocation<T>>,
}

/// A geocoding result
#[derive(Debug, Deserialize)]
pub struct GeoAdminForwardLocation<T>
where
    T: Float,
{
    id: usize,
    pub weight: u32,
    pub attrs: ForwardLocationAttributes<T>,
}

/// Geocoding result attributes
#[derive(Clone, Debug, Deserialize)]
pub struct ForwardLocationAttributes<T> {
    pub detail: String,
    pub origin: String,
    #[serde(rename = "layerBodId")]
    pub layer_bod_id: Option<String>,
    pub rank: u32,
    #[serde(rename = "featureId")]
    pub feature_id: Option<String>,
    pub geom_quadindex: String,
    #[serde(rename = "@geodist")]
    pub geodist: Option<T>,
    pub geom_st_box2d: String,
    pub lat: T,
    pub lon: T,
    pub num: Option<usize>,
    pub x: T,
    pub y: T,
    pub label: String,
    pub zoomlevel: u32,
}

/// The top-level full JSON response returned by a reverse-geocoding request
///
/// See [the documentation](https://api3.geo.admin.ch/services/sdiservices.html#identify-features) for more details
///
///```json
/// {
///     "results": [
///         {
///             "featureId": "1272199_0",
///             "attributes": {
///                 "gdename": "K\u00f6niz",
///                 "strname1": "Seftigenstrasse",
///                 "strname_de": "Seftigenstrasse",
///                 "gdekt": "BE",
///                 "label": "Seftigenstrasse",
///                 "gstat": 1004,
///                 "egid": 1272199,
///                 "dstrid": 1019330,
///                 "strname_fr": null,
///                 "strname_rm": null,
///                 "gdenr": 355,
///                 "plz6": 308400,
///                 "bgdi_created": "22.12.2019",
///                 "plz4": 3084,
///                 "plzname": "Wabern",
///                 "strname_it": null,
///                 "deinr": "264"
///             },
///             "layerBodId": "ch.bfs.gebaeude_wohnungs_register",
///             "layerName": "Register of Buildings and Dwellings",
///             "id": "1272199_0"
///         }
///     ]
/// }
///```
#[derive(Debug, Deserialize)]
pub struct GeoAdminReverseResponse {
    pub results: Vec<GeoAdminReverseLocation>,
}

/// A geocoding result
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

/// Geocoding result attributes
#[derive(Clone, Debug, Deserialize)]
pub struct ReverseLocationAttributes {
    pub gdenr: u32,
    pub gdename: String,
    pub strname1: String,
    pub strname_de: Option<String>,
    pub strname_fr: Option<String>,
    pub strname_rm: Option<String>,
    pub strname_it: Option<String>,
    pub gdekt: String,
    pub label: String,
    pub gstat: u32,
    pub egid: u32,
    pub dstrid: u32,
    pub plz6: u32,
    pub bgdi_created: String,
    pub plz4: u32,
    pub plzname: String,
    pub deinr: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_with_endpoint_forward_test() {
        let geoadmin =
            GeoAdmin::new_with_endpoint("https://api3.geo.admin.ch/rest/services/api/".to_string());
        let address = "Seftigenstrasse 264, 3084 Wabern";
        let res = geoadmin.forward(&address);
        assert_eq!(res.unwrap(), vec![Point::new(2_600_968.75, 1_197_427.0)]);
    }

    #[test]
    fn forward_full_test() {
        let geoadmin = GeoAdmin::new();
        let bbox = InputBounds::new((2_600_967.75, 1_197_426.0), (2_600_969.75, 1_197_428.0));
        let params = GeoAdminParams::new(&"Seftigenstrasse Bern")
            .with_origins("address")
            .with_bbox(&bbox)
            .build();
        let res: GeoAdminForwardResponse<f64> = geoadmin.forward_full(&params).unwrap();
        let result = &res.results[0];
        assert_eq!(result.attrs.label, "Seftigenstrasse 264 <b>3084 Wabern</b>",);
    }

    #[test]
    fn forward_test() {
        let geoadmin = GeoAdmin::new();
        let address = "Seftigenstrasse 264, 3084 Wabern";
        let res = geoadmin.forward(&address);
        assert_eq!(res.unwrap(), vec![Point::new(2_600_968.75, 1_197_427.0)]);
    }

    #[test]
    fn reverse_test() {
        let geoadmin = GeoAdmin::new();
        let p = Point::new(2_600_968.75, 1_197_427.0);
        let res = geoadmin.reverse(&p);
        assert_eq!(
            res.unwrap(),
            Some("Seftigenstrasse 264, 3084 Wabern".to_string()),
        );
    }
}
