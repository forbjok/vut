module commands.command;

interface ICommand {
    int Execute(string[] args);
}

private ICommand[string] commands;

void registerCommand(string command, ICommand implementation) {
    commands[command] = implementation;
}

ICommand getCommand(string command) {
    return commands.get(command, null);
}
