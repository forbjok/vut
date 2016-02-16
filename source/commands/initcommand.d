module commands.initcommand;

import std.stdio;
import std.file;
import std.getopt;

import commands;
import utils.semver;
import services.vutservice;

class InitCommand : ICommand {
    static this() {
        registerCommand("init", new this());
    }

    private void writeUsage(in string command) {
        writefln("Usage: vut %s [version]", command);
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

        string newVersionString;
        if (args.length > 1) {
            // If a version was specified on the commandline, use it
            newVersionString = args[1];
        }
        else {
            // If no version was specified, use default version
            newVersionString = "0.0.0";
        }

        try {
            auto vutService = openVutRoot(getcwd());

            stderr.writefln("An existing version file was found at: %s", vutService.getVersionFilePath());
            return 1;
        }
        catch(NoVutRootFoundException) { }

        try {
            auto vutService = new VutService(getcwd());
            vutService.setVersion(newVersionString);
            vutService.save();

            writefln("Version initialized to %s.", vutService.getVersion());
            return 0;
        }
        catch(InvalidSemanticVersionException ex) {
            stderr.writeln(ex.msg);
            return 1;
        }
    }
}
