import std.exception;
import std.format;
import std.conv;
import std.regex;

private bool isValidPrerelease(string prerelease) {
    auto validatePrerelease = regex(r"^[\w\d\-\.]*$");

    auto m = prerelease.matchFirst(validatePrerelease);
    return !m.empty;
}

private string prefixIfNotEmpty(in string s, in string prefix) {
    if (s.length == 0)
        return s;

    return prefix ~ s;
}

class InvalidSemanticVersionException : Exception {
    this(string versionString) {
        super("Invalid semantic version: %s.".format(versionString));
    }
}

class InvalidPrereleaseException : Exception {
    this(string message) {
        super(message);
    }
}

class NotBumpableException : Exception {
    this(string s) {
        super("No bumpable number found in '%s'.".format(s));
    }
}

class SemanticVersion {
    int major;
    int minor;
    int patch;
    private string _prerelease;
    private string _build;

    @property {
        string prerelease() {
            return this._prerelease;
        }

        string prerelease(string value) {
            if (!value.isValidPrerelease())
                throw new InvalidPrereleaseException("The prerelease string '%s' contains illegal characters.".format(value));

            return this._prerelease = value;
        }
    }

    @property {
        string build() {
            return this._build;
        }

        string build(string value) {
            if (!value.isValidPrerelease())
                throw new InvalidPrereleaseException("The build string '%s' contains illegal characters.".format(value));

            return this._build = value;
        }
    }

    this(int major, int minor, int patch, string prerelease = "", string build = "") {
        this.major = major;
        this.minor = minor;
        this.patch = patch;
        this.prerelease = prerelease;
        this.build = build;
    }

    override string toString() {
        return format("%s.%s.%s%s%s", major, minor, patch, prefixIfNotEmpty(prerelease, "-"), prefixIfNotEmpty(build, "+"));
    }

    // Test: Full semantic version
    unittest {
        auto semVer = new SemanticVersion(1, 2, 3, "beta.6", "build.9");
        assert(semVer.major == 1);
        assert(semVer.minor == 2);
        assert(semVer.patch == 3);
        assert(semVer.prerelease == "beta.6");
        assert(semVer.build == "build.9");
        assert(semVer.toString() == "1.2.3-beta.6+build.9");
    }

    // Test: Major.Minor.Patch w/ prerelease
    unittest {
        auto semVer = new SemanticVersion(1, 2, 3, "beta.6");
        assert(semVer.major == 1);
        assert(semVer.minor == 2);
        assert(semVer.patch == 3);
        assert(semVer.prerelease == "beta.6");
        assert(semVer.build == "");
        assert(semVer.toString() == "1.2.3-beta.6");
    }

    // Test: Major.Minor.Patch only
    unittest {
        auto semVer = new SemanticVersion(1, 2, 3);
        assert(semVer.major == 1);
        assert(semVer.minor == 2);
        assert(semVer.patch == 3);
        assert(semVer.prerelease == "");
        assert(semVer.build == "");
        assert(semVer.toString() == "1.2.3");
    }

    // Test: Major.Minor.Patch w/ build only (no prerelease)
    unittest {
        auto semVer = new SemanticVersion(1, 2, 3, "", "build.9");
        assert(semVer.major == 1);
        assert(semVer.minor == 2);
        assert(semVer.patch == 3);
        assert(semVer.prerelease == "");
        assert(semVer.build == "build.9");
        assert(semVer.toString() == "1.2.3+build.9");
    }
}

SemanticVersion parseSemanticVersion(string versionString) {
    auto parseSemVer = regex(r"^(\d+)\.(\d+)\.(\d+)(?:-([\w\d\-\.]+))?(?:\+([\w\d\-\.]+))?");

    auto m = versionString.matchFirst(parseSemVer);
    if (m.empty)
        throw new InvalidSemanticVersionException(versionString);

    int major = m[1].to!int;
    int minor = m[2].to!int;
    int patch = m[3].to!int;
    string prerelease = m[4];
    string build = m[5];

    return new SemanticVersion(major, minor, patch, prerelease, build);
}

unittest {
    assert(parseSemanticVersion("1.2.3-beta.6+build.9").toString() == "1.2.3-beta.6+build.9");
    assert(parseSemanticVersion("1.2.3-beta-version.6+build-metadata.9").toString() == "1.2.3-beta-version.6+build-metadata.9");
    assert(parseSemanticVersion("1.2.3-beta.6").toString() == "1.2.3-beta.6");
    assert(parseSemanticVersion("1.2.3").toString() == "1.2.3");
    assert(parseSemanticVersion("1.2.3+build.9").toString() == "1.2.3+build.9");
    assertThrown!InvalidSemanticVersionException(parseSemanticVersion("invalid.version"));
}

bool splitNumberedPrerelease(in string prerelease, out string prefix, out int number) {
    auto splitPrerelease = regex(r"([\w\-\.]*?)(\d+)");

    auto m = prerelease.matchFirst(splitPrerelease);
    if (m.empty)
        return false;

    prefix = m[1];
    number = m[2].to!int;

    return true;
}

unittest {
    string prefix;
    int number;

    assert("beta42".splitNumberedPrerelease(prefix, number));
    assert(prefix == "beta");
    assert(number == 42);

    assert("beta.42".splitNumberedPrerelease(prefix, number));
    assert(prefix == "beta.");
    assert(number == 42);

    assert(!"unnumbered.beta".splitNumberedPrerelease(prefix, number));
}

SemanticVersion bumpMajor(SemanticVersion semanticVersion) {
    return new SemanticVersion(semanticVersion.major + 1, 0, 0);
}

SemanticVersion bumpMinor(SemanticVersion semanticVersion) {
    return new SemanticVersion(semanticVersion.major, semanticVersion.minor + 1, 0);
}

SemanticVersion bumpPatch(SemanticVersion semanticVersion) {
    return new SemanticVersion(semanticVersion.major, semanticVersion.minor, semanticVersion.patch + 1);
}

SemanticVersion bumpPrerelease(SemanticVersion semanticVersion) {
    string prereleasePrefix;
    int prereleaseNumber;

    if (!semanticVersion.prerelease.splitNumberedPrerelease(prereleasePrefix, prereleaseNumber))
        throw new NotBumpableException(semanticVersion.prerelease);

    return new SemanticVersion(semanticVersion.major, semanticVersion.minor, semanticVersion.patch, prereleasePrefix ~ (prereleaseNumber + 1).to!string);
}

SemanticVersion bumpBuild(SemanticVersion semanticVersion) {
    string buildPrefix;
    int buildNumber;

    if (!semanticVersion.build.splitNumberedPrerelease(buildPrefix, buildNumber))
        throw new NotBumpableException(semanticVersion.build);

    return new SemanticVersion(semanticVersion.major, semanticVersion.minor, semanticVersion.patch, semanticVersion.prerelease, buildPrefix ~ (buildNumber + 1).to!string);
}

unittest {
    assert(new SemanticVersion(1,2,3).bumpMajor().toString() == "2.0.0");
    assert(new SemanticVersion(1,2,3).bumpMinor().toString() == "1.3.0");
    assert(new SemanticVersion(1,2,3).bumpPatch().toString() == "1.2.4");
    assert(new SemanticVersion(1, 2, 3, "beta.1").bumpPrerelease().toString() == "1.2.3-beta.2");
    assert(new SemanticVersion(1, 2, 3, "beta.1", "build.7").bumpBuild().toString() == "1.2.3-beta.1+build.8");
}
