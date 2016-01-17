import std.stdio;
import std.format;
import std.file;

import commands;
import filelocator;

int main(string[] args)
{
    if (args.length == 1) {
        writeln("No command specified.");
        printUsage();
        return 1;
    }

    // Get command
    auto command = args[1];

    auto commandImplementation = getCommand(command);
    if (commandImplementation is null) {
        stderr.writefln("Unknown command: %s.", command);
        printUsage();
        return 1;
    }

    return commandImplementation.Execute(args[1..$]);
}

void printUsage() {
    writeln("Usage: vut <init|set|get|bump|generate> [--help] [...]");
}
