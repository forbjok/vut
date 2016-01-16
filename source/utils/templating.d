import std.regex;

string replaceTemplateVars(in string input, in string[string] variables) {
    auto findTemplateVars = regex(r"\{\{(?:([^\|\{\}]*)\|)?([\w\d]*)(?:\|([^\}]*))?\}\}");

    string replaceFunction(Captures!(string) m) {
        auto prefix = m[1];
        auto suffix = m[3];
        auto value = variables[m[2]];

        if (value.length == 0)
            return "";

        return prefix ~ value ~ suffix;
    }

    return input.replaceAll!(replaceFunction)(findTemplateVars);
}

unittest {
    immutable string[string] variables = [
        "TheVariable": "42",
        "EmptyVariable": "",
    ];

    assert("BLAH={{TheVariable}};".replaceTemplateVars(variables) == "BLAH=42;");
    assert("BLAH={{.|TheVariable}};".replaceTemplateVars(variables) == "BLAH=.42;");
    assert("BLAH={{.|TheVariable|.}};".replaceTemplateVars(variables) == "BLAH=.42.;");
    assert("BLAH={{TheVariable|.}};".replaceTemplateVars(variables) == "BLAH=42.;");
    assert("BLAH={{EmptyVariable}};".replaceTemplateVars(variables) == "BLAH=;");
    assert("BLAH={{.|EmptyVariable}};".replaceTemplateVars(variables) == "BLAH=;");
    assert("BLAH={{.|EmptyVariable|.}};".replaceTemplateVars(variables) == "BLAH=;");
    assert("BLAH={{EmptyVariable|.}};".replaceTemplateVars(variables) == "BLAH=;");
    assert("BLAH={{TheVariable}};YADA={{EmptyVariable}};".replaceTemplateVars(variables) == "BLAH=42;YADA=;");
    assert("BLAH={{TheVariable}}.{{TheVariable}}.{{TheVariable}};".replaceTemplateVars(variables) == "BLAH=42.42.42;");
    assert("BLAH={{TheVariable}}.{{TheVariable}}.{{TheVariable}}{{-|TheVariable}}{{+|TheVariable}};".replaceTemplateVars(variables) == "BLAH=42.42.42-42+42;");
}
