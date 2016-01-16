import std.stdio;
import std.format;

import commands;

int main(string[] args)
{
	if (args.length == 1) {
		writeln("No command specified.");
		writeln("Usage: vut <command> [...]");
		return 1;
	}

	// Get command
	auto command = args[1];

	auto commandImplementation = getCommand(command);
	commandImplementation.Execute(args[2..$]);

	return 0;
}
