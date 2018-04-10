//! The [OpenCage Geocoding](https://geocoder.opencagedata.com/) provider.
//!
//! Please see the [API documentation](https://geocoder.opencagedata.com/api) for details.
//! Note that rate limits apply to the free tier; the remaining daily quota can be retrieved
//! Using the [`remaining_calls()`](struct.Opencage.html#method.remaining_calls) method.
//! ### A Note on Coordinate Order
//! This provider's API documentation shows all coordinates in `[Latitude, Longitude]` order.
//! However, `Geocoding` requires input `Point` coordinate order as `[Longitude, Latitude]`
//! `(x, y)`, and returns coordinates with that order.
//!
//! ### Example
//!
//! ```
//! use geocoding::{Opencage, Point, Reverse};
//!
//! let oc = Opencage::new("dcdbf0d783374909b3debee728c7cc10".to_string());
//! let p = Point::new(2.12870, 41.40139);
//! let res = oc.reverse(&p);
//! println!("{:?}", res.unwrap());
//! ```
use std::sync::{Arc, Mutex};

use super::num_traits::Float;
use std::collections::HashMap;

use super::Deserialize;
use super::UA_STRING;
use super::reqwest;
use super::{header, Client};

use super::Point;
use super::{Forward, Reverse};

// OpenCage has a custom rate-limit header, indicating remaining calls
header! { (XRatelimitRemaining, "X-RateLimit-Remaining") => [i32] }

/// An instance of the Opencage Geocoding service
pub struct Opencage {
    api_key: String,
    client: Client,
    endpoint: String,
    remaining: Arc<Mutex<Option<i32>>>,
}

impl Opencage {
    /// Create a new OpenCage geocoding instance
    pub fn new(api_key: String) -> Self {
        let mut headers = header::Headers::new();
        headers.set(header::UserAgent::new(UA_STRING));
        let client = Client::builder()
            .default_headers(headers)
            .build()
            .expect("Couldn't build a client!");
        Opencage {
            api_key,
            client,
            endpoint: "https://api.opencagedata.com/geocode/v1/json".to_string(),
            remaining: Arc::new(Mutex::new(None)),
        }
    }
    /// Retrieve the remaining API calls in your daily quota
    ///
    /// Initially, this value is `None`. Any OpenCage API call will update this
    /// value to reflect the remaining quota for the API key. See the [API docs](https://geocoder.opencagedata.com/api#rate-limiting) for details.
    pub fn remaining_calls(&self) -> Option<i32> {
        *self.remaining.lock().unwrap()
    }
}

impl<T> Reverse<T> for Opencage
where
    T: Float,
    for<'de> T: Deserialize<'de>,
{
    /// A reverse lookup of a point. More detail on the format of the
    /// returned `String` can be found [here](https://blog.opencagedata.com/post/99059889253/good-looking-addresses-solving-the-berlin-berlin)
    ///
    /// This method passes the `no_annotations` and `no_record` parameters to the API.
    fn reverse(&self, point: &Point<T>) -> reqwest::Result<String> {
        let mut resp = self.client
            .get(&self.endpoint)
            .query(&[
                (
                    &"q",
                    &format!(
                        "{}, {}",
                        // OpenCage expects lat, lon order
                        (&point.y().to_f64().unwrap().to_string()),
                        &point.x().to_f64().unwrap().to_string()
                    ),
                ),
                (&"key", &self.api_key),
                (&"no_annotations", &String::from("1")),
                (&"no_record", &String::from("1")),
            ])
            .send()?
            .error_for_status()?;
        let res: OpencageResponse<T> = resp.json()?;
        // it's OK to index into this vec, because reverse-geocoding only returns a single result
        let address = &res.results[0];
        let headers = resp.headers().get::<XRatelimitRemaining>().unwrap();
        let mut lock = self.remaining.try_lock();
        if let Ok(ref mut mutex) = lock {
            **mutex = Some(**headers)
        }
        Ok(address.formatted.to_string())
    }
}

impl<T> Forward<T> for Opencage
where
    T: Float,
    for<'de> T: Deserialize<'de>,
{
    /// A forward-geocoding lookup of an address. Please see [the documentation](https://geocoder.opencagedata.com/api#ambiguous-results) for details
    /// of best practices in order to obtain good-quality results.
    ///
    /// This method passes the `no_annotations` and `no_record` parameters to the API.
    fn forward(&self, place: &str) -> reqwest::Result<Vec<Point<T>>> {
        let mut resp = self.client
            .get(&self.endpoint)
            .query(&[
                (&"q", place),
                (&"key", &self.api_key),
                (&"no_annotations", &String::from("1")),
                (&"no_record", &String::from("1")),
            ])
            .send()?
            .error_for_status()?;
        let res: OpencageResponse<T> = resp.json()?;
        let headers = resp.headers().get::<XRatelimitRemaining>().unwrap();
        let mut lock = self.remaining.try_lock();
        if let Ok(ref mut mutex) = lock {
            **mutex = Some(**headers)
        }
        Ok(res.results
            .iter()
            .map(|res| Point::new(res.geometry["lng"], res.geometry["lat"]))
            .collect())
    }
}

/// The top-level full JSON response returned by a forward-geocoding request
///
/// See [the documentation](https://geocoder.opencagedata.com/api#response) for more details
///
///```json
/// {
///   "documentation": "https://geocoder.opencagedata.com/api",
///   "licenses": [
///     {
///       "name": "CC-BY-SA",
///       "url": "http://creativecommons.org/licenses/by-sa/3.0/"
///     },
///     {
///       "name": "ODbL",
///       "url": "http://opendatacommons.org/licenses/odbl/summary/"
///     }
///   ],
///   "rate": {
///     "limit": 2500,
///     "remaining": 2499,
///     "reset": 1523318400
///   },
///   "results": [
///     {
///       "annotations": {
///         "DMS": {
///           "lat": "41° 24' 5.06412'' N",
///           "lng": "2° 7' 43.40064'' E"
///         },
///         "MGRS": "31TDF2717083684",
///         "Maidenhead": "JN11bj56ki",
///         "Mercator": {
///           "x": 236968.295,
///           "y": 5043465.71
///         },
///         "OSM": {
///           "edit_url": "https://www.openstreetmap.org/edit?way=355421084#map=17/41.40141/2.12872",
///           "url": "https://www.openstreetmap.org/?mlat=41.40141&mlon=2.12872#map=17/41.40141/2.12872"
///         },
///         "callingcode": 34,
///         "currency": {
///           "alternate_symbols": [
///
///           ],
///           "decimal_mark": ",",
///           "html_entity": "&#x20AC;",
///           "iso_code": "EUR",
///           "iso_numeric": 978,
///           "name": "Euro",
///           "smallest_denomination": 1,
///           "subunit": "Cent",
///           "subunit_to_unit": 100,
///           "symbol": "€",
///           "symbol_first": 1,
///           "thousands_separator": "."
///         },
///         "flag": "🇪🇸",
///         "geohash": "sp3e82yhdvd7p5x1mbdv",
///         "qibla": 110.53,
///         "sun": {
///           "rise": {
///             "apparent": 1523251260,
///             "astronomical": 1523245440,
///             "civil": 1523249580,
///             "nautical": 1523247540
///           },
///           "set": {
///             "apparent": 1523298360,
///             "astronomical": 1523304180,
///             "civil": 1523300040,
///             "nautical": 1523302080
///           }
///         },
///         "timezone": {
///           "name": "Europe/Madrid",
///           "now_in_dst": 1,
///           "offset_sec": 7200,
///           "offset_string": 200,
///           "short_name": "CEST"
///         },
///         "what3words": {
///           "words": "chins.pictures.passes"
///         }
///       },
///       "bounds": {
///         "northeast": {
///           "lat": 41.4015815,
///           "lng": 2.128952
///         },
///         "southwest": {
///           "lat": 41.401227,
///           "lng": 2.1284918
///         }
///       },
///       "components": {
///         "ISO_3166-1_alpha-2": "ES",
///         "_type": "building",
///         "city": "Barcelona",
///         "city_district": "Sarrià - Sant Gervasi",
///         "country": "Spain",
///         "country_code": "es",
///         "county": "BCN",
///         "house_number": "68",
///         "political_union": "European Union",
///         "postcode": "08017",
///         "road": "Carrer de Calatrava",
///         "state": "Catalonia",
///         "suburb": "les Tres Torres"
///       },
///       "confidence": 10,
///       "formatted": "Carrer de Calatrava, 68, 08017 Barcelona, Spain",
///       "geometry": {
///         "lat": 41.4014067,
///         "lng": 2.1287224
///       }
///     }
///   ],
///   "status": {
///     "code": 200,
///     "message": "OK"
///   },
///   "stay_informed": {
///     "blog": "https://blog.opencagedata.com",
///     "twitter": "https://twitter.com/opencagedata"
///   },
///   "thanks": "For using an OpenCage Data API",
///   "timestamp": {
///     "created_http": "Mon, 09 Apr 2018 12:33:01 GMT",
///     "created_unix": 1523277181
///   },
///   "total_results": 1
/// }
///```
#[derive(Deserialize)]
pub struct OpencageResponse<T>
where
    T: Float,
{
    pub documentation: String,
    pub licenses: Vec<HashMap<String, String>>,
    pub rate: HashMap<String, i32>,
    pub results: Vec<Results<T>>,
    pub status: Status,
    pub stay_informed: HashMap<String, String>,
    pub thanks: String,
    pub timestamp: Timestamp,
    pub total_results: i32,
}

/// A forward geocoding result
#[derive(Deserialize)]
pub struct Results<T>
where
    T: Float,
{
    pub annotations: Option<Annotations<T>>,
    pub bounds: Bounds<T>,
    pub components: HashMap<String, String>,
    pub confidence: i8,
    pub formatted: String,
    pub geometry: HashMap<String, T>,
}

/// Annotations pertaining to the geocoding result
#[derive(Deserialize)]
pub struct Annotations<T>
where
    T: Float,
{
    pub dms: Option<HashMap<String, String>>,
    pub mgrs: Option<String>,
    pub maidenhead: Option<String>,
    pub mercator: Option<HashMap<String, T>>,
    pub osm: Option<HashMap<String, String>>,
    pub callingcode: i16,
    pub currency: Currency,
    pub flag: String,
    pub geohash: String,
    pub qibla: T,
    pub sun: Sun,
    pub timezone: Timezone,
    pub what3words: HashMap<String, String>,
}

/// Currency metadata
#[derive(Deserialize)]
pub struct Currency {
    pub alternate_symbols: Vec<String>,
    pub decimal_mark: String,
    pub html_entity: String,
    pub iso_code: String,
    pub iso_numeric: i16,
    pub name: String,
    pub smallest_denomination: i16,
    pub subunit: String,
    pub subunit_to_unit: i16,
    pub symbol: String,
    pub symbol_first: i16,
    pub thousands_separator: String,
}

/// Sunrise and sunset metadata
#[derive(Deserialize)]
pub struct Sun {
    pub rise: HashMap<String, i64>,
    pub set: HashMap<String, i64>,
}

/// Timezone metadata
#[derive(Deserialize)]
pub struct Timezone {
    pub name: String,
    pub now_in_dst: i16,
    pub offset_sec: i32,
    pub offset_string: i32,
    pub short_name: String,
}

/// HTTP status metadata
#[derive(Deserialize)]
pub struct Status {
    pub message: String,
    pub code: i16,
}

/// Timestamp metadata
// TODO: could this be represented as something less naive?
#[derive(Deserialize)]
pub struct Timestamp {
    pub created_http: String,
    pub created_unix: i64,
}

/// Bounding-box metadata
#[derive(Deserialize)]
pub struct Bounds<T>
where
    T: Float,
{
    pub northeast: HashMap<String, T>,
    pub southwest: HashMap<String, T>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reverse_test() {
        let oc = Opencage::new("dcdbf0d783374909b3debee728c7cc10".to_string());
        let p = Point::new(2.12870, 41.40139);
        let res = oc.reverse(&p);
        assert_eq!(
            res.unwrap(),
            "Carrer de Calatrava, 68, 08017 Barcelona, Spain"
        );
    }
    #[test]
    fn forward_test() {
        let oc = Opencage::new("dcdbf0d783374909b3debee728c7cc10".to_string());
        let address = "Schwabing, München";
        let res = oc.forward(&address);
        assert_eq!(
            res.unwrap(),
            vec![
                Point::new(11.5761796, 48.1599218),
                Point::new(11.57583, 48.1608265),
            ]
        );
    }
}
