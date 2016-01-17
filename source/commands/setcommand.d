import std.stdio;
import std.file;
import std.getopt;

import command;
import semver;
import vutservice;

class SetCommand : ICommand {
    static this() {
        registerCommand("set", new this());
    }

    private void writeUsage(in string command) {
        writefln("Usage: vut %s <version>", command);
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
            writeln("No version specified.");
            writeUsage(args[0]);
            return 1;
        }

        auto versionString = args[1];

        try {
            auto vutService = openVutRoot(getcwd());

            auto semanticVersion = parseSemanticVersion(versionString);
            if (semanticVersion is null) {
                stderr.writefln("Invalid version: %s", versionString);
                return 1;
            }

            auto newVersionString = semanticVersion.toString();

            vutService.setVersion(newVersionString);
            vutService.save();

            writefln("Version set to %s.", newVersionString);
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
    }
}
