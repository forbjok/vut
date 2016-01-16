import std.stdio;
import std.file;
import std.path;

import command;
import filelocator;
import semver;

class InitCommand : ICommand {
    static this() {
        registerCommand("init", new this());
    }

    int Execute(string[] args) {
        SemanticVersion semanticVersion = void;
        if (args.length > 0) {
            // If a version was specified on the commandline, parse it
            auto versionString = args[0];

            semanticVersion = parseSemanticVersion(versionString);
            if (semanticVersion is null) {
                writefln("Invalid version: %s", versionString);
                return 1;
            }
        }
        else {
            // If no version was specified, use default version
            semanticVersion = new SemanticVersion(0, 0, 0);
        }

        auto versionFile = locateFileInPathOrParent(getcwd(), "VERSION");

        if (versionFile != null) {
            writefln("An existing version file was found at: %s", versionFile);
            return 1;
        }

        auto newVersionString = semanticVersion.toString();

        // Write new version to file
        std.file.write(buildPath(getcwd(), "VERSION"), cast(void[]) newVersionString);

        writefln("Version initialized to %s.", newVersionString);

        return 0;
    }
}
