import std.stdio;
import std.file;
import std.conv;
import std.getopt;

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

    private void writeUsage(in string command) {
        writefln("Usage: vut %s <major|minor|patch|prerelease|build>", command);
    }

    int Execute(string[] args) {
        try {
            // Parse arguments
            auto getoptResult = getopt(args);

            if (getoptResult.helpWanted) {
                // If user wants help, give it to them
                writeUsage(args[0]);
                return 1;
            }
        }
        catch(Exception ex) {
            // If there is an error parsing arguments, print it
            writeln(ex.msg);
            return 1;
        }

        if (args.length == 1) {
            writeln("No version part specified.");
            writeUsage(args[0]);
            return 1;
        }

        auto versionPart = args[1];

        try {
            auto vutService = openVutRoot(getcwd());

            auto semanticVersion = parseSemanticVersion(vutService.getVersion());
            SemanticVersion newVersion = void;

            switch(versionPart.to!VersionPart) {
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
                    stderr.writefln("Invalid version part '%s'.", versionPart);
                    return 1;
            }

            auto newVersionString = newVersion.toString();

            vutService.setVersion(newVersionString);
            vutService.save();

            writefln("Version bumped to %s.", newVersionString);
            return 0;
        }
        catch(NoVutRootFoundException) {
            stderr.writeln("No version file found.");
            return 1;
        }
        catch(InvalidSemanticVersionException ex) {
            stderr.writeln(ex.msg);
            return 1;
        }
        catch(NotBumpableException ex) {
            stderr.writeln(ex.msg);
            return 1;
        }
    }
}
