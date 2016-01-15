import std.stdio;
import std.regex;
import std.conv;
import std.format;

SemanticVersion bumpMajor(SemanticVersion semanticVersion) {
    return new SemanticVersion(semanticVersion.major + 1, 0, 0);
}

SemanticVersion bumpMinor(SemanticVersion semanticVersion) {
    return new SemanticVersion(semanticVersion.major, semanticVersion.minor + 1, 0);
}

SemanticVersion bumpPatch(SemanticVersion semanticVersion) {
    return new SemanticVersion(semanticVersion.major, semanticVersion.minor, semanticVersion.patch + 1);
}

unittest {
    assert(new SemanticVersion(1,2,3).bumpMajor().toString() == "2.0.0");
    assert(new SemanticVersion(1,2,3).bumpMinor().toString() == "1.3.0");
    assert(new SemanticVersion(1,2,3).bumpPatch().toString() == "1.2.4");
}

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

    this(string versionString) {
        auto parseSemVer = ctRegex!(`^(\d+)\.(\d+)\.(\d+)(?:-([\w\d]+))?(?:\+([\w\d]+))?`);

        auto m = versionString.matchFirst(parseSemVer);
        if (m.empty) {
            writeln("Fail regex");
        }

        major = m[1].to!int;
        minor = m[2].to!int;
        patch = m[3].to!int;
        prerelease = m[4];
        build = m[5];

        debug writefln("Major = %s, Minor = %s, Patch = %s, Prerelease = %s, Build = %s", major, minor, patch, prerelease, build);
    }

    override string toString() {
        return format("%s.%s.%s%s%s", major, minor, patch, prefixIfNotEmpty(prerelease, "-"), prefixIfNotEmpty(build, "+"));
    }

    // Test: Full semantic version
    unittest {
        auto semVer = new SemanticVersion("1.2.3-beta6+build9");
        assert(semVer.major == 1);
        assert(semVer.minor == 2);
        assert(semVer.patch == 3);
        assert(semVer.prerelease == "beta6");
        assert(semVer.build == "build9");
        assert(semVer.toString() == "1.2.3-beta6+build9");
    }

    // Test: Major.Minor.Patch w/ prerelease
    unittest {
        auto semVer = new SemanticVersion("1.2.3-beta6");
        assert(semVer.major == 1);
        assert(semVer.minor == 2);
        assert(semVer.patch == 3);
        assert(semVer.prerelease == "beta6");
        assert(semVer.build == "");
        assert(semVer.toString() == "1.2.3-beta6");
    }

    // Test: Major.Minor.Patch only
    unittest {
        auto semVer = new SemanticVersion("1.2.3");
        assert(semVer.major == 1);
        assert(semVer.minor == 2);
        assert(semVer.patch == 3);
        assert(semVer.prerelease == "");
        assert(semVer.build == "");
        assert(semVer.toString() == "1.2.3");
    }

    // Test: Major.Minor.Patch w/ build only (no prerelease)
    unittest {
        auto semVer = new SemanticVersion("1.2.3+build9");
        assert(semVer.major == 1);
        assert(semVer.minor == 2);
        assert(semVer.patch == 3);
        assert(semVer.prerelease == "");
        assert(semVer.build == "build9");
        assert(semVer.toString() == "1.2.3+build9");
    }
}
