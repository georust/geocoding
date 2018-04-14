//! This crate provides forward– and reverse-geocoding functionality for Rust.
//! Over time, a variety of providers will be added. Each provider may implement one or both
//! of the `Forward` and `Reverse` traits, which provide forward– and reverse-geocoding methods.
//!
//! Note that for the `reverse` method, the return type is simply `String`,
//! As this is the lowest common denominator reverse-geocoding result.
//! Individual providers may implement additional methods, which return more
//! finely-structured and/or extensive data, and enable more specific query tuning.
//! ### A note on Coordinate Order
//! While individual providers may specify coordinates in either `[Longitude, Latitude]` **or**
//! `[Latitude, Longitude`] order,
//! `Geocoding` **always** requires `Point` data in `[Longitude, Latitude]` (`x, y`) order,
//! and returns data in that order.
//!
static UA_STRING: &'static str = "Rust-Geocoding";
extern crate geo;
pub use geo::Point;

extern crate num_traits;
use num_traits::Float;

#[macro_use]
extern crate serde_derive;

extern crate serde;
use serde::Deserialize;

extern crate reqwest;
use reqwest::{header, Client};

#[macro_use]
extern crate hyper;

// The OpenCage geocoding provider
pub mod opencage;
pub use opencage::Opencage;

/// Reverse-geocode a coordinate.
///
/// This trait represents the most simple and minimal implementation
/// available from a given geocoding provider: an address formatted as a String.
///
/// Examples
///
/// ```
/// use geocoding::Point;
/// use geocoding::Reverse;
/// use geocoding::Opencage;
/// let p = Point::new(2.12870, 41.40139);
/// let oc = Opencage::new("dcdbf0d783374909b3debee728c7cc10".to_string());
/// let res = oc.reverse(&p);
/// assert_eq!(
///     res.unwrap(),
///     "Carrer de Calatrava, 68, 08017 Barcelona, Spain"
/// );
/// ```
pub trait Reverse<T>
where
    T: Float,
{
    // NOTE TO IMPLEMENTERS: Point coordinates are lon, lat (x, y)
    // You may have to provide these coordinates in reverse order,
    // depending on the provider's requirements (see e.g. OpenCage)
    fn reverse(&self, point: &Point<T>) -> reqwest::Result<String>;
}

/// Forward-geocode a coordinate.
///
/// This trait represents the most simple and minimal implementation available
/// from a given geocoding provider: It returns a `Vec` of zero or more `Points`.
///
/// Examples
///
/// ```
/// use geocoding::Point;
/// use geocoding::Forward;
/// use geocoding::Opencage;
/// let p = Point::new(2.12870, 41.40139);
/// let oc = Opencage::new("dcdbf0d783374909b3debee728c7cc10".to_string());
/// let address = "Schwabing, München";
/// let res = oc.forward(&address);
/// assert_eq!(
///     res.unwrap(),
///     vec![
///         Point::new(11.5761796, 48.1599218),
///     ]
/// );
/// ```
pub trait Forward<T>
where
    T: Float,
{
    // NOTE TO IMPLEMENTERS: while returned provider point data may not be in
    // lon, lat (x, y) order, Geocoding requires this order in its output Point
    // data. Please pay attention when using returned data to construct Points
    fn forward(&self, address: &str) -> reqwest::Result<Vec<Point<T>>>;
}
