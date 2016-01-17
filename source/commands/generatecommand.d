import std.stdio;
import std.file;

import command;
import semver;
import vutservice;

class GenerateCommand : ICommand {
    static this() {
        auto instance = new this();

        registerCommand("generate", instance);
        registerCommand("gen", instance);
    }

    int Execute(string[] args) {
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
