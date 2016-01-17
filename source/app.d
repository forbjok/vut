import std.algorithm;
import std.array;
import std.range;
import std.stdio;
import std.getopt;
import std.path;

import commands;

int main(string[] args)
{
    try {
        /* Concatenate the executable path with all subsequent arguments up to
           the first one that does not start with optionChar. (normally "-")

           This is required to prevent getopt from parsing options that are
           intended for the command. */
        auto options = chain(args.takeExactly(1), args[1..$].until!(a => !a.startsWith(optionChar))).array();

        // Parse arguments
        auto getoptResult = getopt(options);

        if (getoptResult.helpWanted) {
            // If user wants help, give it to them
            writeUsage(args[0]);
            return 0;
        }
    }
    catch(Exception ex) {
        // If there is an error parsing arguments, print it
        writeln(ex.msg);
        return 1;
    }

    if (args.length == 1) {
        writeln("No command specified.");
        writeUsage(args[0]);
        return 1;
    }

    // Get command
    auto command = args[1];

    auto commandImplementation = getCommand(command);
    if (commandImplementation is null) {
        stderr.writefln("Unknown command: %s.", command);
        writeUsage(args[0]);
        return 1;
    }

    return commandImplementation.Execute(args[1..$]);
}

void writeUsage(in string executable) {
    writefln("Usage: %s <init|set|get|bump|generate> [--help] [...]", executable.baseName());
}
