interface ICommand {
    int Execute(string[] args);
}

ICommand[string] commands;

void registerCommand(string command, ICommand implementation) {
    commands[command] = implementation;
}

ICommand getCommand(string command) {
    return commands[command];
}
