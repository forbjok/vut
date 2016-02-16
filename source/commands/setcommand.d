module commands.setcommand;

import std.stdio;
import std.file;
import std.getopt;

import commands;
import utils.semver;
import services.vutservice;

class SetCommand : ICommand {
    static this() {
        registerCommand("set", new this());
    }

    private void writeUsage(in string command) {
        writefln("Usage: vut %s <version>", command);
    }

    int Execute(string[] args) {
        string build;

        try {
            // Parse arguments
            auto getoptResult = getopt(args,
                std.getopt.config.bundling,
                "build", &build);

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

        try {
            auto vutService = openVutRoot(getcwd());

            string newVersionString;
            if (build.length > 0) {
                auto semanticVersion = vutService.getVersion().parseSemanticVersion();
                semanticVersion.build = build;

                newVersionString = semanticVersion.toString();
            }
            else {
                if (args.length == 1) {
                    writeln("No version specified.");
                    writeUsage(args[0]);
                    return 1;
                }

                newVersionString = args[1];
            }

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
        catch(InvalidPrereleaseException ex) {
            stderr.writeln(ex.msg);
            return 1;
        }
    }
}
