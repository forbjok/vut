import core.exception;

interface ICommand {
    int Execute(string[] args);
}

private ICommand[string] commands;

void registerCommand(string command, ICommand implementation) {
    commands[command] = implementation;
}

ICommand getCommand(string command) {
    try {
        return commands[command];
    }
    catch(RangeError) {
        return null;
    }
}
