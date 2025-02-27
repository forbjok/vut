use std::{str::FromStr, sync::LazyLock};

use regex::Regex;

static REGEX_PARSE_SEMVER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^(\d+)\.(\d+)\.(\d+)(?:-([\w\d\-\.]+))?(?:\+([\w\d\-\.]+))?"#).unwrap());
static REGEX_SPLIT_NUMBERED_PRERELEASE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"([\w\-\.]*?)(\d+)"#).unwrap());

/// Structure representing a SemVer compliant version.
#[derive(Clone, Debug)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: Option<String>,
    pub build: Option<String>,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32, prerelease: Option<&str>, build: Option<&str>) -> Self {
        Self {
            major,
            minor,
            patch,
            prerelease: prerelease.map(|s| s.to_owned()),
            build: build.map(|s| s.to_owned()),
        }
    }

    /// Bump the major version.
    pub fn bump_major(&self) -> Self {
        Self {
            major: self.major + 1,
            minor: 0,
            patch: 0,
            prerelease: None,
            build: self.build.clone(),
        }
    }

    /// Bump the minor version.
    pub fn bump_minor(&self) -> Self {
        Self {
            major: self.major,
            minor: self.minor + 1,
            patch: 0,
            prerelease: None,
            build: self.build.clone(),
        }
    }

    /// Bump the patch version.
    pub fn bump_patch(&self) -> Self {
        Self {
            major: self.major,
            minor: self.minor,
            patch: self.patch + 1,
            prerelease: None,
            build: self.build.clone(),
        }
    }

    /// Bump the number of a numbered prerelease string.
    pub fn bump_prerelease(&self) -> Self {
        if let Some(prerelease) = &self.prerelease {
            if let Some((prefix, number)) = split_numbered_prerelease(prerelease) {
                Self {
                    major: self.major,
                    minor: self.minor,
                    patch: self.patch,
                    prerelease: Some(format!("{}{}", prefix, number + 1)),
                    build: self.build.clone(),
                }
            } else {
                self.clone()
            }
        } else {
            self.clone()
        }
    }

    /// Bump the number of a numbered build string.
    pub fn bump_build(&self) -> Self {
        if let Some(build) = &self.build {
            if let Some((prefix, number)) = split_numbered_prerelease(build) {
                Self {
                    major: self.major,
                    minor: self.minor,
                    patch: self.patch,
                    prerelease: self.prerelease.to_owned(),
                    build: Some(format!("{}{}", prefix, number + 1)),
                }
            } else {
                self.clone()
            }
        } else {
            self.clone()
        }
    }
}

impl FromStr for Version {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cap = REGEX_PARSE_SEMVER
            .captures(s)
            .ok_or_else(|| format!("Invalid version string: {}", s))?;

        Ok(Self {
            major: cap[1].parse().unwrap(),
            minor: cap[2].parse().unwrap(),
            patch: cap[3].parse().unwrap(),
            prerelease: cap.get(4).map(|m| m.as_str().to_owned()),
            build: cap.get(5).map(|m| m.as_str().to_owned()),
        })
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}{}{}",
            self.major,
            self.minor,
            self.patch,
            self.prerelease
                .as_ref()
                .map_or_else(|| "".to_owned(), |p| format!("-{}", p)),
            self.build.as_ref().map_or_else(|| "".to_owned(), |b| format!("+{}", b))
        )
    }
}

/// Split a prerelease string with a number at the end into separate string prefix and number components.
pub fn split_numbered_prerelease(s: &str) -> Option<(&str, u32)> {
    match REGEX_SPLIT_NUMBERED_PRERELEASE.captures(s) {
        Some(cap) => {
            let prefix = cap.get(1).unwrap().as_str();
            let number = cap.get(2).unwrap().as_str().parse().unwrap();

            Some((prefix, number))
        }
        _ => None,
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
        assert_eq!(
            "1.2.3-beta.6+build.9".parse::<Version>().unwrap().to_string(),
            "1.2.3-beta.6+build.9"
        );
        assert_eq!(
            "1.2.3-beta-version.6+build-metadata.9"
                .parse::<Version>()
                .unwrap()
                .to_string(),
            "1.2.3-beta-version.6+build-metadata.9"
        );
        assert_eq!("1.2.3-beta.6".parse::<Version>().unwrap().to_string(), "1.2.3-beta.6");
        assert_eq!("1.2.3".parse::<Version>().unwrap().to_string(), "1.2.3");
        assert_eq!("1.2.3+build.9".parse::<Version>().unwrap().to_string(), "1.2.3+build.9");
    }

    #[test]
    /// Test bumps
    fn test_bumps() {
        assert_eq!(Version::new(1, 2, 3, None, None).bump_major().to_string(), "2.0.0");
        assert_eq!(Version::new(1, 2, 3, None, None).bump_minor().to_string(), "1.3.0");
        assert_eq!(Version::new(1, 2, 3, None, None).bump_patch().to_string(), "1.2.4");

        assert_eq!(
            Version::new(1, 2, 3, Some("beta.1"), None)
                .bump_prerelease()
                .to_string(),
            "1.2.3-beta.2"
        );
        assert_eq!(
            Version::new(1, 2, 3, Some("beta.1"), Some("build.7"))
                .bump_build()
                .to_string(),
            "1.2.3-beta.1+build.8"
        );
    }
}
