use std::borrow::Cow;
use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref REGEX_PARSE_SEMVER: Regex = Regex::new(r#"^(\d+)\.(\d+)\.(\d+)(?:-([\w\d\-\.]+))?(?:\+([\w\d\-\.]+))?"#).unwrap();
    static ref REGEX_SPLIT_NUMBERED_PRERELEASE: Regex = regex::Regex::new(r#"([\w\-\.]*?)(\d+)"#).unwrap();
}

/// Structure representing a SemVer compliant version.
#[derive(Clone, Debug)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: String,
    pub build: String,
}

impl Version {
    pub fn new() -> Self {
        Self {
            major: 0,
            minor: 0,
            patch: 0,
            prerelease: "".to_owned(),
            build: "".to_owned(),
        }
    }

    pub fn new_rel(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            prerelease: "".to_owned(),
            build: "".to_owned(),
        }
    }

    pub fn new_pre(major: u32, minor: u32, patch: u32, prerelease: &str, build: &str) -> Self {
        Self {
            major,
            minor,
            patch,
            prerelease: prerelease.to_owned(),
            build: build.to_owned(),
        }
    }

    /// Bump the major version.
    pub fn bump_major(&self) -> Self {
        Self {
            major: self.major + 1,
            minor: 0,
            patch: 0,
            prerelease: "".to_owned(),
            build: "".to_owned(),
        }
    }

    /// Bump the minor version.
    pub fn bump_minor(&self) -> Self {
        Self {
            major: self.major,
            minor: self.minor + 1,
            patch: 0,
            prerelease: "".to_owned(),
            build: "".to_owned(),
        }
    }

    /// Bump the patch version.
    pub fn bump_patch(&self) -> Self {
        Self {
            major: self.major,
            minor: self.minor,
            patch: self.patch + 1,
            prerelease: "".to_owned(),
            build: "".to_owned(),
        }
    }

    /// Bump the number of a numbered prerelease string.
    pub fn bump_prerelease(&self) -> Self {
        if let Some((prefix, number)) = split_numbered_prerelease(&self.prerelease) {
            Self {
                major: self.major,
                minor: self.minor,
                patch: self.patch,
                prerelease: format!("{}{}", prefix, number + 1),
                build: "".to_owned(),
            }
        } else {
            self.clone()
        }
    }

    /// Bump the number of a numbered build string.
    pub fn bump_build(&self) -> Self {
        if let Some((prefix, number)) = split_numbered_prerelease(&self.build) {
            Self {
                major: self.major,
                minor: self.minor,
                patch: self.patch,
                prerelease: self.prerelease.to_owned(),
                build: format!("{}{}", prefix, number + 1),
            }
        } else {
            self.clone()
        }
    }
}

impl FromStr for Version {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cap = REGEX_PARSE_SEMVER.captures(s).unwrap();

        Ok(Self {
            major: (&cap[1]).parse().unwrap(),
            minor: (&cap[2]).parse().unwrap(),
            patch: (&cap[3]).parse().unwrap(),
            prerelease: cap.get(4).map_or("", |m| m.as_str()).to_owned(),
            build: cap.get(5).map_or("", |m| m.as_str()).to_owned(),
        })
    }
}

impl ToString for Version {
    fn to_string(&self) -> String {
        format!("{}.{}.{}{}{}", self.major, self.minor, self.patch, prefix_if_not_empty(&self.prerelease, "-"), prefix_if_not_empty(&self.build, "+"))
    }
}

/// Add a prefix to the specified string only if it is not empty.
fn prefix_if_not_empty<'a>(s: &'a str, prefix: &str) -> Cow<'a, str> {
    if s.is_empty() {
        return Cow::Borrowed(s);
    }

    Cow::Owned(format!("{}{}", prefix, s))
}

/// Split a prerelease string with a number at the end into separate string prefix and number components.
fn split_numbered_prerelease(s: &str) -> Option<(&str, u32)> {
    if let Some(cap) = REGEX_SPLIT_NUMBERED_PRERELEASE.captures(s) {

        let prefix = cap.get(1).unwrap().as_str();
        let number = cap.get(2).unwrap().as_str().parse().unwrap();

        Some((prefix, number))
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    /// Test the split_numbered_prerelease utility function
    fn test_split_numbered_prerelease() {
        assert_eq!(split_numbered_prerelease("beta42"), Some(("beta", 42)));
        assert_eq!(split_numbered_prerelease("beta.42"), Some(("beta.", 42)));
        assert_eq!(split_numbered_prerelease("unnumbered.beta"), None);
    }

    #[test]
    /// Test to make sure that versions round-trip accurately when parsed and converted back to a string
    fn test_parse() {
        assert_eq!("1.2.3-beta.6+build.9".parse::<Version>().unwrap().to_string(), "1.2.3-beta.6+build.9");
        assert_eq!("1.2.3-beta-version.6+build-metadata.9".parse::<Version>().unwrap().to_string(), "1.2.3-beta-version.6+build-metadata.9");
        assert_eq!("1.2.3-beta.6".parse::<Version>().unwrap().to_string(), "1.2.3-beta.6");
        assert_eq!("1.2.3".parse::<Version>().unwrap().to_string(), "1.2.3");
        assert_eq!("1.2.3+build.9".parse::<Version>().unwrap().to_string(), "1.2.3+build.9");
    }

    #[test]
    /// Test bumps
    fn test_bumps() {
        assert_eq!(Version::new_rel(1, 2, 3).bump_major().to_string(), "2.0.0");
        assert_eq!(Version::new_rel(1, 2, 3).bump_minor().to_string(), "1.3.0");
        assert_eq!(Version::new_rel(1, 2, 3).bump_patch().to_string(), "1.2.4");

        assert_eq!(Version::new_pre(1, 2, 3, "beta.1", "").bump_prerelease().to_string(), "1.2.3-beta.2");
        assert_eq!(Version::new_pre(1, 2, 3, "beta.1", "build.7").bump_build().to_string(), "1.2.3-beta.1+build.8");
    }
}
