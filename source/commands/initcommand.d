import std.stdio;
import std.file;

import command;
import semver;
import vutservice;

class InitCommand : ICommand {
    static this() {
        registerCommand("init", new this());
    }

    int Execute(string[] args) {
        SemanticVersion semanticVersion = void;
        if (args.length > 1) {
            // If a version was specified on the commandline, parse it
            auto versionString = args[1];

            semanticVersion = parseSemanticVersion(versionString);
            if (semanticVersion is null) {
                stderr.writefln("Invalid version: %s", versionString);
                return 1;
            }
        }
        else {
            // If no version was specified, use default version
            semanticVersion = new SemanticVersion(0, 0, 0);
        }

        auto newVersionString = semanticVersion.toString();

        try {
            auto vutService = openVutRoot(getcwd());

            stderr.writefln("An existing version file was found at: %s", vutService.getVersionFilePath());
            return 1;
        }
        catch(NoVutRootFoundException) { }

        auto vutService = new VutService(getcwd());
        vutService.setVersion(newVersionString);
        vutService.save();

        writefln("Version initialized to %s.", newVersionString);
        return 0;
    }
}
