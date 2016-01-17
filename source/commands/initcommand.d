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
