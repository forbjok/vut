module commands.generatecommand;

import std.stdio;
import std.file;
import std.getopt;

import commands;
import utils.semver;
import services.vutservice;

class GenerateCommand : ICommand {
    static this() {
        auto instance = new this();

        registerCommand("generate", instance);
        registerCommand("gen", instance);
    }

    private void writeUsage(in string command) {
        writefln("Usage: vut %s", command);
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

        try {
            auto vutService = openVutRoot(getcwd());

            write("Generating templates... ");
            vutService.processTemplates();

            writeln("Done.");
            return 0;
        }
        catch(NoVutRootFoundException) {
            stderr.writeln("No version file found.");
            return 1;
        }
    }
}
