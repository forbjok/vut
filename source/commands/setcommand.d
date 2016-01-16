import std.stdio;
import std.file;

import command;
import filelocator;
import semver;

class SetCommand : ICommand {
    static this() {
        registerCommand("set", new this());
    }

    int Execute(string[] args) {
        if (args.length == 0) {
            writeln("No version specified.");
            writeln("Usage: vut set <version>");
            return 1;
        }

        auto versionString = args[0];

        auto versionFile = locateFileInPathOrParent(getcwd(), "VERSION");

        if (versionFile == null) {
            writeln("No version file found.");
            return 1;
        }

        auto semanticVersion = parseSemanticVersion(versionString);
        if (semanticVersion is null) {
            writefln("Invalid version: %s", versionString);
            return 1;
        }

        auto newVersionString = semanticVersion.toString();

        // Write new version to file
        std.file.write(versionFile, cast(void[]) newVersionString);

        writefln("Version set to %s.", newVersionString);

        return 0;
    }
}
