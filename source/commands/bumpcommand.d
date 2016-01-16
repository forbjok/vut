import std.stdio;
import std.file;
import std.conv;

import command;
import semver;
import vutservice;

class SetCommand : ICommand {
    static this() {
        registerCommand("bump", new this());
    }

    enum VersionPart {
        major,
        minor,
        patch,
        prerelease,
        build,
    }

    int Execute(string[] args) {
        if (args.length == 0) {
            writeln("No version part specified.");
            writeln("Usage: vut bump <major|minor|patch|prerelease>");
            return 1;
        }

        auto versionPart = args[0].to!VersionPart;

        try {
            auto vutService = openVutRoot(getcwd());

            auto semanticVersion = parseSemanticVersion(vutService.getVersion());
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
                case VersionPart.prerelease:
                    newVersion = semanticVersion.bumpPrerelease();
                    break;
                case VersionPart.build:
                    newVersion = semanticVersion.bumpBuild();
                    break;
                default:
                    writefln("Invalid version part '%s'.", args[0]);
                    return 1;
            }

            auto newVersionString = newVersion.toString();

            vutService.setVersion(newVersionString);
            vutService.save();

            writefln("Version bumped to %s.", newVersionString);
        }
        catch(NoVutRootFoundException) {
            writeln("No version file found.");
            return 1;
        }

        return 0;
    }
}
