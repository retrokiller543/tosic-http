//! # Route
//!
//! A structured representation of a route

use crate::route::path::Path;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::ops::Add;

#[derive(Debug, PartialEq, Eq, Clone)]
/// # Route
///
/// A structured representation of a route as a sequence of [`PathSegment`]
///
pub struct Route {
    path: Vec<PathSegment>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// # PathSegment
///
/// A segment of a [`Route`]
///
/// A path segment can be:
/// - a static string
/// - a parameter
/// - a wildcard
/// - a wildcard deep
pub enum PathSegment {
    Static(Cow<'static, str>),
    Parameter(Cow<'static, str>),
    Wildcard,
    WildcardDeep,
}

impl Display for PathSegment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Static(string) => write!(f, "{}", string),
            Self::Parameter(param) => write!(f, "{}", param),
            Self::Wildcard => write!(f, "*"),
            Self::WildcardDeep => write!(f, "**"),
        }
    }
}

impl Route {
    /// Create a new route
    pub fn new(path: &str) -> Self {
        Self {
            path: Path::parse(path),
        }
    }

    /// Get the segments of the route
    pub fn segments(&self) -> &[PathSegment] {
        &self.path
    }

    /// Check if the request path matches the route
    pub fn is_match(&self, request_path: &str) -> Option<BTreeMap<String, String>> {
        let request_segments = request_path
            .split('/')
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>();
        let mut params = BTreeMap::new();

        if self.matches_segments(&request_segments, &mut params) {
            Some(params)
        } else {
            None
        }
    }

    /// Check if the request path matches the route
    fn matches_segments(
        &self,
        request_segments: &[&str],
        params: &mut BTreeMap<String, String>,
    ) -> bool {
        let route_iter = self.path.iter();
        let mut request_iter = request_segments.iter();

        for route_segment in route_iter {
            match route_segment {
                PathSegment::Static(route_str) => {
                    if let Some(req_segment) = request_iter.next() {
                        if route_str != req_segment {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                PathSegment::Parameter(param_name) => {
                    if let Some(req_segment) = request_iter.next() {
                        params.insert(param_name.to_string(), req_segment.to_string());
                    } else {
                        return false;
                    }
                }
                PathSegment::Wildcard => {
                    if request_iter.next().is_none() {
                        return false;
                    }
                }
                PathSegment::WildcardDeep => {
                    let remaining: Vec<_> = request_iter.cloned().collect();
                    params.insert("wildcard_deep".to_string(), remaining.join("/"));
                    return true;
                }
            }
        }

        request_iter.next().is_none()
    }
}

impl Ord for Route {
    fn cmp(&self, other: &Self) -> Ordering {
        self.path.len().cmp(&other.path.len())
    }
}

impl PartialOrd for Route {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Add for Route {
    type Output = Route;

    fn add(self, other: Route) -> Route {
        let mut combined_path = self.path;
        combined_path.extend(other.path);
        Route {
            path: combined_path,
        }
    }
}

impl<'a> IntoIterator for &'a Route {
    type Item = &'a PathSegment;
    type IntoIter = std::slice::Iter<'a, PathSegment>;

    fn into_iter(self) -> Self::IntoIter {
        self.path.iter()
    }
}

impl FromIterator<PathSegment> for Route {
    fn from_iter<I: IntoIterator<Item = PathSegment>>(iter: I) -> Self {
        Route {
            path: iter.into_iter().collect(),
        }
    }
}
