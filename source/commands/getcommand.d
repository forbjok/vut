import std.algorithm;
import std.stdio;
import std.file;
import std.format;
import std.string;
import std.json;
import std.getopt;

import command;
import semver;
import templating;
import vutservice;

class GetCommand : ICommand {
    static this() {
        registerCommand("get", new this());
    }

    private void writeUsage(in string command) {
        writefln("Usage: vut %s [--format=<pattern>]", command);
    }

    int Execute(string[] args) {
        string format;

        try {
            // Parse arguments
            auto getoptResult = getopt(args,
                std.getopt.config.bundling,
                "f|format", &format);

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
            auto variables = vutService.getVersionVariables();

            if (format.length > 0) {
                // A format argument was passed.
                // Process the specified format string using standard
                // template replacement and write it to stdout.
                try {
                    auto output = format.replaceTemplateVars(variables);

                    write(output);
                    return 0;
                }
                catch(VariableNotFoundException ex) {
                    stderr.writeln(ex.msg);
                    return 1;
                }
            }

            // No special output format was specified.
            // Default to JSON.

            string[string] jsonVariables;

            // Convert first letter of all variables to lowercase JSON-style
            foreach(key, value; variables) {
                auto jsonKey = "%s%s".format(key[0].toLower(), key[1..$]);
                jsonVariables[jsonKey] = value;
            }

            auto json = new JSONValue(jsonVariables);

            write(json.toPrettyString());
            return 0;
        }
        catch(NoVutRootFoundException) {
            stderr.writeln("No version file found.");
            return 1;
        }
    }
}
