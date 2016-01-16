import std.stdio;
import std.file;

import command;
import semver;
import vutservice;

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

        try {
            auto vutService = openVutRoot(getcwd());

            auto semanticVersion = parseSemanticVersion(versionString);
            if (semanticVersion is null) {
                writefln("Invalid version: %s", versionString);
                return 1;
            }

            auto newVersionString = semanticVersion.toString();

            vutService.setVersion(newVersionString);
            vutService.save();

            writefln("Version set to %s.", newVersionString);
        }
        catch(NoVutRootFoundException) {
            writeln("No version file found.");
            return 1;
        }

        return 0;
    }
}
