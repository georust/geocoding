//! This crate provides forward– and reverse-geocoding functionality for Rust.
//! Over time, a variety of providers will be added. Each provider may implement one or both
//! of the `Forward` and `Reverse` traits, which provide forward– and reverse-geocoding methods.
//!
//! Note that for the `reverse` method, the return type is simply `String`,
//! As this is the lowest common denominator reverse-geocoding result. Individual providers
//! may implement additional methods, which return more finely-structured and/or
//! extensive data, and enable more specific query tuning.

extern crate num_traits;
use num_traits::{Float, ToPrimitive};

#[macro_use]
extern crate serde_derive;

extern crate serde;
use serde::Deserialize;

extern crate reqwest;
use reqwest::Client;

// The OpenCage geocoding provider
pub mod opencage;
pub use opencage::Opencage;

/// A primitive type which holds `x` and `y` position information
#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Coordinate<T>
where
    T: Float,
{
    pub x: T,
    pub y: T,
}

/// A single Point in 2D space.
///
/// Points can be created using the `new(x, y)` constructor, or from a `Coordinate` or pair of points.
///
/// ```
/// use geocoding::{Point, Coordinate};
/// let p1: Point<f64> = (0., 1.).into();
/// let c = Coordinate{ x: 10., y: 20.};
/// let p2: Point<f64> = c.into();
/// ```
#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Point<T>(pub Coordinate<T>)
where
    T: Float;

impl<T: Float> From<Coordinate<T>> for Point<T> {
    fn from(x: Coordinate<T>) -> Point<T> {
        Point(x)
    }
}

impl<T: Float> From<(T, T)> for Point<T> {
    fn from(coords: (T, T)) -> Point<T> {
        Point::new(coords.0, coords.1)
    }
}

impl<T> Point<T>
where
    T: Float + ToPrimitive,
{
    /// Creates a new point.
    ///
    /// ```
    /// use geocoding::Point;
    ///
    /// let p = Point::new(1.234, 2.345);
    ///
    /// assert_eq!(p.x(), 1.234);
    /// assert_eq!(p.y(), 2.345);
    /// ```
    pub fn new(x: T, y: T) -> Point<T> {
        Point(Coordinate { x: x, y: y })
    }
    /// Returns the x/horizontal component of the point.
    ///
    /// ```
    /// use geocoding::Point;
    ///
    /// let p = Point::new(1.234, 2.345);
    ///
    /// assert_eq!(p.x(), 1.234);
    /// ```
    pub fn x(&self) -> T {
        self.0.x
    }
    /// Returns the y/vertical component of the point.
    ///
    /// ```
    /// use geocoding::Point;
    ///
    /// let p = Point::new(1.234, 2.345);
    ///
    /// assert_eq!(p.y(), 2.345);
    /// ```
    pub fn y(&self) -> T {
        self.0.y
    }
}

/// Reverse-geocode a coordinate.
///
/// Note that this trait represents the most simple and minimal implementation of
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
/// Note that this trait represents the most simple and minimal implementation available
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

