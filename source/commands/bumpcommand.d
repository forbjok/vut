import std.stdio;
import std.file;
import std.conv;

import command;
import filelocator;
import semver;

class SetCommand : ICommand {
    static this() {
        registerCommand("bump", new this());
    }

    enum VersionPart {
        major,
        minor,
        patch
    }

    int Execute(string[] args) {
        if (args.length == 0) {
            writeln("No version part specified.");
            writeln("Usage: vut bump <major|minor|patch>");
            return 1;
        }

        auto versionPart = args[0].to!VersionPart;

        auto versionFile = locateFileInPathOrParent(getcwd(), "VERSION");

        if (versionFile == null) {
            writeln("No version file found.");
            return 1;
        }

        auto versionString = readText(versionFile);
        auto semanticVersion = parseSemanticVersion(versionString);
        if (semanticVersion is null) {
            writefln("Invalid version in file: %s", versionString);
            return 1;
        }

        SemanticVersion newVersion = void;

        switch(versionPart) {
            case VersionPart.major:
                newVersion = semanticVersion.bumpMajor();
                break;
            case VersionPart.minor:
                newVersion = semanticVersion.bumpMinor();
                break;
            case VersionPart.patch:
                newVersion = semanticVersion.bumpPatch();
                break;
            default:
                writefln("Invalid version part: %s", args[0]);
                return 1;
        }

        auto newVersionString = newVersion.toString();

        // Write new version to file
        std.file.write(versionFile, cast(void[]) newVersionString);

        writefln("Version bumped to %s.", newVersionString);

        return 0;
    }
}
