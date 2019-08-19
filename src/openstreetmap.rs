//! The [OpenStreetMap Nominatim](https://nominatim.org/) provider.
//!
//! Geocoding methods are implemented on the [`Openstreetmap`](struct.Openstreetmap.html) struct.
//! Please see the [API documentation](https://nominatim.org/release-docs/develop/) for details.
//!
//! While OpenStreetMap's Nominatim API is free, see the [Nominatim Usage Policy](https://operations.osmfoundation.org/policies/nominatim/)
//! for details on usage requirements, including a maximum of 1 request per second.
//!
//! ### Example
//!
//! ```
//! use geocoding::{Openstreetmap, Forward, Point};
//!
//! let osm = Openstreetmap::new();
//! let address = "Schwabing, München";
//! let res = oc.forward(&address);
//! assert_eq!(res.unwrap(), vec![Point::new(11.5761796, 48.1599218)]);
//! ```
use crate::Deserialize;
use crate::Point;
use crate::UA_STRING;
use crate::{Client, HeaderMap, HeaderValue, USER_AGENT};
use crate::{Forward, Reverse};
use failure::Error;
use num_traits::Float;

/// An instance of the Openstreetmap geocoding service
pub struct Openstreetmap {
    client: Client,
    endpoint: String,
}

/// An instance of a parameter builder for Openstreetmap geocoding
pub struct OpenstreetmapParams<'a> {
    query: &'a str,
    addressdetails: &'a bool,
    viewbox: Option<&'a [f64; 4]>,
}

impl<'a> OpenstreetmapParams<'a> {
    /// Create a new OpenStreetMap parameter builder
    /// # Example:
    ///
    /// ```
    /// use geocoding::{Openstreetmap, OpenstreetmapParams, Point};
    ///
    /// let params = OpenstreetmapParams::new(&"UCL CASA")
    ///     .with_addressdetails(&true)
    ///     .with_viewbox(&[
    ///         -0.13806939125061035,
    ///         51.51989264641164,
    ///         -0.13427138328552246,
    ///         51.52319711775629,
    ///     ])
    ///     .build();
    /// ```
    pub fn new(query: &'a str) -> OpenstreetmapParams<'a> {
        OpenstreetmapParams {
            query,
            addressdetails: &false,
            viewbox: None,
        }
    }

    /// Set the `addressdetails` property
    pub fn with_addressdetails(&mut self, addressdetails: &'a bool) -> &mut Self {
        self.addressdetails = addressdetails;
        self
    }

    /// Set the `viewbox` property
    pub fn with_viewbox(&mut self, viewbox: &'a [f64; 4]) -> &mut Self {
        self.viewbox = Some(viewbox);
        self
    }

    /// Build and return an instance of OpenstreetmapParams
    pub fn build(&self) -> OpenstreetmapParams<'a> {
        OpenstreetmapParams {
            query: self.query,
            addressdetails: self.addressdetails,
            viewbox: self.viewbox,
        }
    }
}

impl Openstreetmap {
    /// Create a new Openstreetmap geocoding instance
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(UA_STRING));
        let client = Client::builder()
            .default_headers(headers)
            .build()
            .expect("Couldn't build a client!");
        Openstreetmap {
            client,
            endpoint: "https://nominatim.openstreetmap.org/".to_string(),
        }
    }

    /// A forward-geocoding lookup of an address, returning a full detailed response
    ///
    /// Accepts an [`OpenstreetmapParams`](struct.OpenstreetmapParams.html) struct for specifying
    /// options, including whether to include address details in the response and whether to filter
    /// by a bounding box.
    ///
    /// Please see [the documentation](https://nominatim.org/release-docs/develop/api/Search/) for details.
    ///
    /// This method passes the `format` parameter to the API.
    ///
    /// # Examples
    ///
    /// ```
    /// use geocoding::{Openstreetmap, OpenstreetmapParams, Point};
    ///
    /// let osm = Openstreetmap::new();
    /// let params = OpenstreetmapParams::new(&"UCL CASA")
    ///     .with_addressdetails(&true)
    ///     .with_viewbox(&[
    ///         -0.13806939125061035,
    ///         51.51989264641164,
    ///         -0.13427138328552246,
    ///         51.52319711775629,
    ///     ])
    ///     .build();
    /// let res: OpenstreetmapResponse<f64> = osm.forward_full(&params).unwrap();
    /// let result = res.features[0].properties.clone();
    /// assert_eq!(
    ///     result.display_name,
    ///     "UCL, 188, Tottenham Court Road, Holborn, Bloomsbury, London Borough of Camden, London, Greater London, England, W1T 7PQ, UK",
    /// );
    /// ```
    pub fn forward_full<T>(
        &self,
        params: &OpenstreetmapParams,
    ) -> Result<OpenstreetmapResponse<T>, Error>
    where
        T: Float,
        for<'de> T: Deserialize<'de>,
    {
        let format = String::from("geojson");
        let addressdetails = String::from(if *params.addressdetails { "1" } else { "0" });
        // For lifetime issues
        let viewbox;

        let mut query = vec![
            (&"q", params.query),
            (&"format", &format),
            (&"addressdetails", &addressdetails),
        ];

        if let Some(vb) = params.viewbox {
            viewbox = vb
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>()
                .join(",");
            query.push((&"viewbox", &viewbox));
        }

        let mut resp = self
            .client
            .get(&format!("{}search", self.endpoint))
            .query(&query)
            .send()?
            .error_for_status()?;
        let res: OpenstreetmapResponse<T> = resp.json()?;
        Ok(res)
    }
}

impl<T> Forward<T> for Openstreetmap
where
    T: Float,
    for<'de> T: Deserialize<'de>,
{
    /// A forward-geocoding lookup of an address. Please see [the documentation](https://nominatim.org/release-docs/develop/api/Search/) for details.
    ///
    /// This method passes the `format` parameter to the API.
    fn forward(&self, place: &str) -> Result<Vec<Point<T>>, Error> {
        let mut resp = self
            .client
            .get(&format!("{}search", self.endpoint))
            .query(&[(&"q", place), (&"format", &String::from("geojson"))])
            .send()?
            .error_for_status()?;
        let res: OpenstreetmapResponse<T> = resp.json()?;
        Ok(res
            .features
            .iter()
            .map(|res| Point::new(res.geometry.coordinates.0, res.geometry.coordinates.1))
            .collect())
    }
}

impl<T> Reverse<T> for Openstreetmap
where
    T: Float,
    for<'de> T: Deserialize<'de>,
{
    /// A reverse lookup of a point. More detail on the format of the
    /// returned `String` can be found [here](https://nominatim.org/release-docs/develop/api/Reverse/)
    ///
    /// This method passes the `format` parameter to the API.
    fn reverse(&self, point: &Point<T>) -> Result<String, Error> {
        let mut resp = self
            .client
            .get(&format!("{}reverse", self.endpoint))
            .query(&[
                (&"lon", &point.x().to_f64().unwrap().to_string()),
                (&"lat", &point.y().to_f64().unwrap().to_string()),
                (&"format", &String::from("geojson")),
            ])
            .send()?
            .error_for_status()?;
        let res: OpenstreetmapResponse<T> = resp.json()?;
        let address = &res.features[0];
        Ok(address.properties.display_name.to_string())
    }
}

/// The top-level full GeoJSON response returned by a forward-geocoding request
///
/// See [the documentation](https://nominatim.org/release-docs/develop/api/Search/#geojson) for more details
///
///```json
///{
///  "type": "FeatureCollection",
///  "licence": "Data © OpenStreetMap contributors, ODbL 1.0. https://osm.org/copyright",
///  "features": [
///    {
///      "type": "Feature",
///      "properties": {
///        "place_id": 263681481,
///        "osm_type": "way",
///        "osm_id": 355421084,
///        "display_name": "68, Carrer de Calatrava, les Tres Torres, Sarrià - Sant Gervasi, Barcelona, BCN, Catalonia, 08017, Spain",
///        "place_rank": 30,
///        "category": "building",
///        "type": "apartments",
///        "importance": 0.7409999999999999,
///        "address": {
///          "house_number": "68",
///          "road": "Carrer de Calatrava",
///          "suburb": "les Tres Torres",
///          "city_district": "Sarrià - Sant Gervasi",
///          "city": "Barcelona",
///          "county": "BCN",
///          "state": "Catalonia",
///          "postcode": "08017",
///          "country": "Spain",
///          "country_code": "es"
///        }
///      },
///      "bbox": [
///        2.1284918,
///        41.401227,
///        2.128952,
///        41.4015815
///      ],
///      "geometry": {
///        "type": "Point",
///        "coordinates": [
///          2.12872241167437,
///          41.40140675
///        ]
///      }
///    }
///  ]
///}
///```
#[derive(Debug, Deserialize)]
pub struct OpenstreetmapResponse<T>
where
    T: Float,
{
    r#type: String,
    licence: String,
    features: Vec<OpenstreetmapResult<T>>,
}

/// A geocoding result
#[derive(Debug, Deserialize)]
pub struct OpenstreetmapResult<T>
where
    T: Float,
{
    pub r#type: String,
    pub properties: ResultProperties,
    pub bbox: (T, T, T, T),
    pub geometry: ResultGeometry<T>,
}

/// Geocoding result properties
#[derive(Clone, Debug, Deserialize)]
pub struct ResultProperties {
    place_id: u64,
    osm_type: String,
    osm_id: u64,
    display_name: String,
    place_rank: u64,
    category: String,
    r#type: String,
    importance: f64,
    address: Option<AddressDetails>,
}

/// Address details in the result object
#[derive(Clone, Debug, Deserialize)]
pub struct AddressDetails {
    city: Option<String>,
    city_district: Option<String>,
    construction: Option<String>,
    continent: Option<String>,
    country: Option<String>,
    country_code: Option<String>,
    house_number: Option<String>,
    neighbourhood: Option<String>,
    postcode: Option<String>,
    public_building: Option<String>,
    state: Option<String>,
    suburb: Option<String>,
}

/// A geocoding result geometry
#[derive(Debug, Deserialize)]
pub struct ResultGeometry<T>
where
    T: Float,
{
    r#type: String,
    coordinates: (T, T),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn forward_full_test() {
        let osm = Openstreetmap::new();
        let params = OpenstreetmapParams::new(&"UCL CASA")
            .with_addressdetails(&true)
            .with_viewbox(&[
                -0.13806939125061035,
                51.51989264641164,
                -0.13427138328552246,
                51.52319711775629,
            ])
            .build();
        let res: OpenstreetmapResponse<f64> = osm.forward_full(&params).unwrap();
        let result = res.features[0].properties.clone();
        assert_eq!(
            result.display_name,
            "UCL, 188, Tottenham Court Road, Holborn, Bloomsbury, London Borough of Camden, London, Greater London, England, W1T 7PQ, UK",
        );
        assert_eq!(result.address.unwrap().city.unwrap(), "London");
    }

    #[test]
    fn forward_test() {
        let osm = Openstreetmap::new();
        let address = "Schwabing, München";
        let res = osm.forward(&address);
        assert_eq!(res.unwrap(), vec![Point::new(11.5761796, 48.1599218)]);
    }

    #[test]
    fn reverse_test() {
        let osm = Openstreetmap::new();
        let p = Point::new(2.12870, 41.40139);
        let res = osm.reverse(&p);
        assert_eq!(
            res.unwrap(),
            "68, Carrer de Calatrava, les Tres Torres, Sarrià - Sant Gervasi, Barcelona, BCN, CAT, 08017, España",
        );
    }
}
