//! The [GeoAdmin] (https://api3.geo.admin.ch) provider.
//!
//! Based on the [Search API] (https://api3.geo.admin.ch/services/sdiservices.html#search)
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
use crate::Deserialize;
use crate::InputBounds;
use crate::Point;
use crate::UA_STRING;
use crate::{Client, HeaderMap, HeaderValue, USER_AGENT};
use crate::{Forward, Reverse};
use failure::Error;
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
            searchtext: searchtext,
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
        GeoAdmin {
            client,
            endpoint: endpoint.to_string(),
        }
    }

    /// A forward-geocoding search of a location, returning a full detailed response
    ///
    /// Accepts an [`GeoAdminParams`](struct.GeoAdminParams.html) struct for specifying
    /// options, including what origins to response and whether to filter
    /// by a bounding box.
    ///
    /// Please see [the documentation](https://api3.geo.admin.ch/services/sdiservices.html#search/) for details.
    ///
    /// This method passes the `format` parameter to the API.
    ///
    /// # Examples
    ///
    /// ```
    /// use geocoding::{GeoAdmin, InputBounds, Point};
    /// use geocoding::geoadmin::{GeoAdminParams, GeoAdminResponse};
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
    /// let res: GeoAdminResponse<f64> = geoadmin.forward_full(&params).unwrap();
    /// let result = &res.results[0];
    /// assert_eq!(
    ///     result.attrs.label,
    ///     "Seftigenstrasse 264 <b>3084 Wabern</b>",
    /// );
    /// ```
    pub fn forward_full<T>(&self, params: &GeoAdminParams<T>) -> Result<GeoAdminResponse<T>, Error>
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
        let res: GeoAdminResponse<T> = resp.json()?;
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
    /// A forward-geocoding lookup of an address. Please see [the documentation](https://api3.geo.admin.ch/services/sdiservices.html#search/) for details.
    ///
    /// This method passes the `type`,  `origins`, `limit` and `sr` parameter to the API.
    fn forward(&self, place: &str) -> Result<Vec<Point<T>>, Error> {
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
        let res: GeoAdminResponse<T> = resp.json()?;
        Ok(res
            .results
            .iter()
            .map(|res| Point::new(res.attrs.lon, res.attrs.lat))
            .collect())
    }
}

impl<T> Reverse<T> for GeoAdmin
where
    T: Float,
    for<'de> T: Deserialize<'de>,
{
    /// A reverse lookup of a point. More detail on the format of the
    /// returned `String` can be found [here](https://nominatim.org/release-docs/develop/api/Reverse/)
    ///
    /// This method passes the `format` parameter to the API.
    fn reverse(&self, point: &Point<T>) -> Result<Option<String>, Error> {
        let mut resp = self
            .client
            .get(&format!("{}SearchServer", self.endpoint))
            .query(&[
                (
                    &"bbox",
                    &String::from(InputBounds::new(
                        Point::new(
                            point.x().to_f64().unwrap() - 500.,
                            point.y().to_f64().unwrap() - 500.,
                        ),
                        Point::new(
                            point.x().to_f64().unwrap() + 500.,
                            point.y().to_f64().unwrap() + 500.,
                        ),
                    )),
                ),
                (&"type", &String::from("locations")),
                (&"origins", &String::from("address")),
                (&"limit", &String::from("1")),
                (&"sr", &String::from("2056")),
            ])
            .send()?
            .error_for_status()?;
        println!("{:?}", resp.url());
        let res: GeoAdminResponse<T> = resp.json()?;
        if res.results.len() > 0 {
            Ok(Some(res.results[0].attrs.label.to_string()))
        } else {
            Ok(None)
        }
    }
}
/// The top-level full JSON response returned by a forward-geocoding request
///
/// See [the documentation](https://api3.geo.admin.ch/services/sdiservices.html#search/) for more details
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
pub struct GeoAdminResponse<T>
where
    T: Float,
{
    pub results: Vec<GeoAdminLocation<T>>,
}

/// A geocoding result
#[derive(Debug, Deserialize)]
pub struct GeoAdminLocation<T>
where
    T: Float,
{
    id: usize,
    pub weight: u32,
    pub attrs: LocationAttributes<T>,
}

/// Geocoding result attributes
#[derive(Clone, Debug, Deserialize)]
pub struct LocationAttributes<T> {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_with_endpoint_forward_test() {
        let geoadmin =
            GeoAdmin::new_with_endpoint("https://api3.geo.admin.ch/rest/services/api/".to_string());
        let address = "Seftigenstrasse 264, 3084 Wabern";
        let res = geoadmin.forward(&address);
        assert_eq!(
            res.unwrap(),
            vec![Point::new(7.451352119445801, 46.92793655395508)]
        );
    }

    #[test]
    fn forward_full_test() {
        let geoadmin = GeoAdmin::new();
        let bbox = InputBounds::new((2600967.75, 1197426.0), (2600969.75, 1197428.0));
        let params = GeoAdminParams::new(&"Seftigenstrasse Bern")
            .with_origins("address")
            .with_bbox(&bbox)
            .build();
        let res: GeoAdminResponse<f64> = geoadmin.forward_full(&params).unwrap();
        let result = &res.results[0];
        assert_eq!(result.attrs.label, "Seftigenstrasse 264 <b>3084 Wabern</b>",);
    }

    #[test]
    fn forward_test() {
        let geoadmin = GeoAdmin::new();
        let address = "Seftigenstrasse 264, 3084 Wabern";
        let res = geoadmin.forward(&address);
        assert_eq!(
            res.unwrap(),
            vec![Point::new(7.451352119445801, 46.92793655395508)]
        );
    }

    #[test]
    fn reverse_test() {
        let geoadmin = GeoAdmin::new();
        let p = Point::new(2600968.75, 1197427.0);
        let res = geoadmin.reverse(&p);
        assert_eq!(
            res.unwrap(),
            Some("Seftigenstrasse 264 <b>3084 Wabern</b>".to_string()),
        );
    }
}
