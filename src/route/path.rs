//! No idea why this is a struct, but it is and it works

use super::PathSegment;
use std::borrow::Cow;

/// Path parser
pub struct Path;

impl Path {
    /// Parse a path
    pub fn parse(path: &str) -> Vec<PathSegment> {
        path.split('/')
            .filter(|segment| !segment.is_empty())
            .map(|segment| {
                if segment.starts_with('{') && segment.ends_with('}') {
                    PathSegment::Parameter(Cow::Owned(segment[1..segment.len() - 1].to_string()))
                } else if segment == "*" {
                    PathSegment::Wildcard
                } else if segment == "**" {
                    PathSegment::WildcardDeep
                } else {
                    PathSegment::Static(Cow::Owned(segment.to_string()))
                }
            })
            .collect()
    }
}
