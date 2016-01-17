import std.conv;
import std.path;
import std.file;
import std.format;

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
        auto prerelease = this.semanticVersion.prerelease;
        string prereleasePrefix;
        int prereleaseNumber;
        prerelease.splitNumberedPrerelease(prereleasePrefix, prereleaseNumber);

        auto build = this.semanticVersion.build;
        string buildPrefix;
        int buildNumber;
        build.splitNumberedPrerelease(buildPrefix, buildNumber);

        string[string] variables = [
            "FullVersion": this.semanticVersion.toString(),
            "Major": this.semanticVersion.major.to!string,
            "Minor": this.semanticVersion.minor.to!string,
            "Patch": this.semanticVersion.patch.to!string,
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

            // Read template
            auto text = readText(templateFile);

            // Perform replacements
            text = text.replaceTemplateVars(variables);

            // Write output file
            std.file.write(outputFile, cast(void[]) text);
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
