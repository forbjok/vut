import std.conv;
import std.path;
import std.file;
import std.format;
import std.stdio;
import std.utf;

import filelocator;
import semver;
import templating;

immutable string versionFilename = "VERSION";
immutable string templateExtension = ".vutemplate";

class VutService {
    private string rootPath;
    private SemanticVersion semanticVersion;

    this(string rootPath) {
        this.rootPath = rootPath.absolutePath();
    }

    string getVersionFilePath() {
        return buildPath(rootPath, versionFilename);
    }

    void read() {
        auto versionString = readText(this.getVersionFilePath());
        this.semanticVersion = parseSemanticVersion(versionString);
    }

    void save() {
        // Write new version to file
        std.file.write(this.getVersionFilePath(), cast(void[]) this.semanticVersion.toString());

        this.processTemplates();
    }

    string getVersion() {
        return this.semanticVersion.toString();
    }

    void setVersion(string versionString) {
        this.semanticVersion = parseSemanticVersion(versionString);
    }

    string[string] getVersionVariables() {
        auto major = this.semanticVersion.major.to!string;
        auto minor = this.semanticVersion.minor.to!string;
        auto patch = this.semanticVersion.patch.to!string;

        auto prerelease = this.semanticVersion.prerelease;
        string prereleasePrefix;
        int prereleaseNumber;
        prerelease.splitNumberedPrerelease(prereleasePrefix, prereleaseNumber);

        auto build = this.semanticVersion.build;
        string buildPrefix;
        int buildNumber;
        build.splitNumberedPrerelease(buildPrefix, buildNumber);

        auto majorMinorPatchPrerelease = "%s.%s.%s%s".format(major, minor, patch, prerelease.length > 0 ? "-" ~ prerelease : "");
        auto majorMinorPatch = "%s.%s.%s".format(major, minor, patch);
        auto majorMinor = "%s.%s".format(major, minor);

        string[string] variables = [
            "FullVersion": this.semanticVersion.toString(),
            "Version": majorMinorPatchPrerelease,
            "MajorMinorPatch": majorMinorPatch,
            "MajorMinor": majorMinor,
            "Major": major,
            "Minor": minor,
            "Patch": patch,
            "Prerelease": prerelease,
            "PrereleasePrefix": prereleasePrefix,
            "PrereleaseNumber": prereleaseNumber.to!string,
            "Build": build,
            "BuildPrefix": buildPrefix,
            "BuildNumber": buildNumber.to!string,
        ];

        return variables;
    }

    void processTemplates() {
        auto templateFiles = locateTemplates(rootPath, templateExtension);

        auto variables = this.getVersionVariables();

        foreach(string templateFile; templateFiles) {
            auto outputFile = templateFile.stripExtension();
            auto templateFilename = templateFile.baseName();

            try {
                // Read template
                auto text = readText(templateFile);

                // Perform replacements
                text = text.replaceTemplateVars(variables);

                // Write output file
                std.file.write(outputFile, cast(void[]) text);
            }
            catch(Exception ex) {
                stderr.writefln("%s: %s", templateFilename, ex.msg);
                continue;
            }
        }
    }
}

class NoVutRootFoundException : Exception {
    this(string path) {
        super(format("No Vut root found from '%s'", path));
    }
}

VutService openVutRoot(string path) {
    auto versionFile = locateFileInPathOrParent(path, versionFilename);
    if (versionFile is null) {
        throw new NoVutRootFoundException(path);
    }

    auto vutService = new VutService(versionFile.dirName());
    vutService.read();

    return vutService;
}
