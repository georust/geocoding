//! This crate provides forward– and reverse-geocoding functionality for Rust.
//! Over time, a variety of providers will be added. Each provider may implement one or both
//! of the `Forward` and `Reverse` traits, which provide forward– and reverse-geocoding methods.
//!
//! Note that for the `reverse` method, the return type is simply `String`,
//! As this is the lowest common denominator reverse-geocoding result.
//! Individual providers may implement additional methods, which return more
//! finely-structured and/or extensive data, and enable more specific query tuning.
//! ### A note on Coordinate Order
//! While individual providers may specify coordinates in `lon, lat` or `lat, lon` order,
//! `Geocoding` **always** specifies `Point` data in `lon, lat` order,
//! and returns data in that order.

extern crate geo;
use geo::{Point};

extern crate num_traits;
use num_traits::{Float};

#[macro_use]
extern crate serde_derive;

extern crate serde;
use serde::Deserialize;

extern crate reqwest;
use reqwest::Client;

// The OpenCage geocoding provider
pub mod opencage;
pub use opencage::Opencage;

/// Reverse-geocode a coordinate.
///
/// This trait represents the most simple and minimal implementation of
/// functionality available from a given geocoding provider: an address formatted as a String.
///
/// Note that individual providers may specify coordinate order, which will vary between
/// implementations.
pub trait Reverse<T>
where
    T: Float,
{
    fn reverse(&self, point: &Point<T>) -> reqwest::Result<String>;
}

/// Forward-geocode a coordinate.
///
/// This trait represents the most simple and minimal implementation available
/// from a given geocoding provider; for reverse lookups. It returns a `Vec` of `Points`.
///
/// Note that individual providers may specify coordinate order, which will vary between
/// implementations.
pub trait Forward<T>
where
    T: Float,
{
    fn forward(&self, address: &str) -> reqwest::Result<Vec<Point<T>>>;
}

