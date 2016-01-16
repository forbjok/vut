import std.stdio;
import std.file;

import command;
import filelocator;

class InitCommand : ICommand {
    static this() {
        registerCommand("init", new this());
    }

    int Execute(string[] args) {
        writeln("Init!");

        auto configFile = locateFileInPathOrParent(getcwd(), "VERSION");

        if (configFile == null) {
            writeln("No version file found.");
            return 1;
        }

        return 0;
    }
}
