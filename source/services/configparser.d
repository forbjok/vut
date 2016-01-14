import std.file;
import std.json;

struct Config {
    int Arnvall;
}

class ConfigParser {
    Config* ParseConfigFile(string filename) {
        auto json = readText(filename);

        return new Config();
    }
}
