import std.file;
import std.json;

struct Config {
    int Arnvall;
}

Config* parseConfigFile(string filename) {
    auto json = readText(filename);

    return new Config();
}
