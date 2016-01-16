import std.format;
import std.conv;
import std.regex;

class SemanticVersion {
    int major;
    int minor;
    int patch;
    string prerelease;
    string build;

    private string prefixIfNotEmpty(string s, string prefix) {
        if (s.length == 0)
            return "";

        return prefix ~ s;
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
        auto semVer = new SemanticVersion(1, 2, 3, "beta6", "build9");
        assert(semVer.major == 1);
        assert(semVer.minor == 2);
        assert(semVer.patch == 3);
        assert(semVer.prerelease == "beta6");
        assert(semVer.build == "build9");
        assert(semVer.toString() == "1.2.3-beta6+build9");
    }

    // Test: Major.Minor.Patch w/ prerelease
    unittest {
        auto semVer = new SemanticVersion(1, 2, 3, "beta6");
        assert(semVer.major == 1);
        assert(semVer.minor == 2);
        assert(semVer.patch == 3);
        assert(semVer.prerelease == "beta6");
        assert(semVer.build == "");
        assert(semVer.toString() == "1.2.3-beta6");
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
        auto semVer = new SemanticVersion(1, 2, 3, "", "build9");
        assert(semVer.major == 1);
        assert(semVer.minor == 2);
        assert(semVer.patch == 3);
        assert(semVer.prerelease == "");
        assert(semVer.build == "build9");
        assert(semVer.toString() == "1.2.3+build9");
    }
}

SemanticVersion parseSemanticVersion(string versionString) {
    auto parseSemVer = regex(r"^(\d+)\.(\d+)\.(\d+)(?:-([\w\d]+))?(?:\+([\w\d]+))?");

    auto m = versionString.matchFirst(parseSemVer);
    if (m.empty)
        return null;

    int major = m[1].to!int;
    int minor = m[2].to!int;
    int patch = m[3].to!int;
    string prerelease = m[4];
    string build = m[5];

    debug import std.stdio;
    debug writefln("Major = %s, Minor = %s, Patch = %s, Prerelease = %s, Build = %s", major, minor, patch, prerelease, build);

    return new SemanticVersion(major, minor, patch, prerelease, build);
}

unittest {
    assert(parseSemanticVersion("1.2.3-beta6+build9").toString() == "1.2.3-beta6+build9");
    assert(parseSemanticVersion("1.2.3-beta6").toString() == "1.2.3-beta6");
    assert(parseSemanticVersion("1.2.3").toString() == "1.2.3");
    assert(parseSemanticVersion("1.2.3+build9").toString() == "1.2.3+build9");
    assert(parseSemanticVersion("invalid.version") is null);
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
    auto splitPrerelease = regex(r"([\w]*)(\d+)");

    auto m = semanticVersion.prerelease.matchFirst(splitPrerelease);
    if (m.empty)
        throw new Exception(format("No bumpable number found in prerelease '%s'", semanticVersion.prerelease));

    auto prereleaseNumber = m[2].to!int;

    return new SemanticVersion(semanticVersion.major, semanticVersion.minor, semanticVersion.patch, m[1] ~ (prereleaseNumber + 1).to!string);
}

SemanticVersion bumpBuild(SemanticVersion semanticVersion) {
    auto splitBuild = regex(r"([\w]*)(\d+)");

    auto m = semanticVersion.build.matchFirst(splitBuild);
    if (m.empty)
        throw new Exception(format("No bumpable number found in build '%s'", semanticVersion.build));

    auto buildNumber = m[2].to!int;

    return new SemanticVersion(semanticVersion.major, semanticVersion.minor, semanticVersion.patch, semanticVersion.prerelease, m[1] ~ (buildNumber + 1).to!string);
}

unittest {
    assert(new SemanticVersion(1,2,3).bumpMajor().toString() == "2.0.0");
    assert(new SemanticVersion(1,2,3).bumpMinor().toString() == "1.3.0");
    assert(new SemanticVersion(1,2,3).bumpPatch().toString() == "1.2.4");
    assert(new SemanticVersion(1, 2, 3, "beta1").bumpPrerelease().toString() == "1.2.3-beta2");
    assert(new SemanticVersion(1, 2, 3, "beta1", "build7").bumpBuild().toString() == "1.2.3-beta1+build8");
}
