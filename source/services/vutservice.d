import std.conv;
import std.path;
import std.file;
import std.format;

import filelocator;
import semver;
import templating;

class VutService {
    immutable string versionFilename = "VERSION";
    immutable string templateExtension = ".vutemplate";

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

    void processTemplates() {
        auto templateFiles = locateTemplates(rootPath, templateExtension);

        string[string] variables = [
            "FullVersion": this.semanticVersion.toString(),
            "Major": this.semanticVersion.major.to!string,
            "Minor": this.semanticVersion.minor.to!string,
            "Patch": this.semanticVersion.patch.to!string,
            "Prerelease": this.semanticVersion.prerelease,
            "Build": this.semanticVersion.build,
        ];

        foreach(string templateFile; templateFiles) {
            auto outputFile = templateFile.stripExtension();

            debug import std.stdio;
            debug auto templateFilename = templateFile.baseName();
            debug auto outputFilename = outputFile.baseName();
            debug writefln("%s => %s", templateFilename, outputFilename);

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
    auto versionFile = locateFileInPathOrParent(path, "VERSION");
    if (versionFile is null) {
        throw new NoVutRootFoundException(path);
    }

    auto vutService = new VutService(versionFile.dirName());
    vutService.read();

    return vutService;
}
