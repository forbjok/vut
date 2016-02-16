module utils.templating;

import std.exception;
import std.format;
import std.regex;

class VariableNotFoundException : Exception {
    this(string variableName) {
        super("Variable not found: %s.".format(variableName));
    }
}

string replaceTemplateVars(in string input, in string[string] variables) {
    auto findTemplateVars = regex(r"\{\{(?:\|([^\|]*)\|)?([\w\d]*)(?:\|([^\|]*)\|)?\}\}");

    string replaceFunction(Captures!(string) m) {
        auto prefix = m[1];
        auto suffix = m[3];
        auto variableName = m[2];
        auto value = variables.get(variableName, null);

        if (value is null)
            throw new VariableNotFoundException(variableName);

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
    assert("BLAH={{|.|TheVariable}};".replaceTemplateVars(variables) == "BLAH=.42;");
    assert("BLAH={{|.|TheVariable|.|}};".replaceTemplateVars(variables) == "BLAH=.42.;");
    assert("BLAH={{TheVariable|.|}};".replaceTemplateVars(variables) == "BLAH=42.;");
    assert("BLAH={{EmptyVariable}};".replaceTemplateVars(variables) == "BLAH=;");
    assert("BLAH={{|.|EmptyVariable}};".replaceTemplateVars(variables) == "BLAH=;");
    assert("BLAH={{|.|EmptyVariable|.|}};".replaceTemplateVars(variables) == "BLAH=;");
    assert("BLAH={{EmptyVariable|.|}};".replaceTemplateVars(variables) == "BLAH=;");
    assert("BLAH={{TheVariable}};YADA={{EmptyVariable}};".replaceTemplateVars(variables) == "BLAH=42;YADA=;");
    assert("BLAH={{TheVariable}}.{{TheVariable}}.{{TheVariable}};".replaceTemplateVars(variables) == "BLAH=42.42.42;");
    assert("BLAH={{TheVariable}}.{{TheVariable}}.{{TheVariable}}{{|-|TheVariable}}{{|+|TheVariable}};".replaceTemplateVars(variables) == "BLAH=42.42.42-42+42;");
    assertThrown!VariableNotFoundException("BLAH={{NonExistentVariable}};".replaceTemplateVars(variables));
    assert("BLAH={{|prefix|TheVariable}};".replaceTemplateVars(variables) == "BLAH=prefix42;");
    assert("BLAH={{TheVariable|suffix|}};".replaceTemplateVars(variables) == "BLAH=42suffix;");
}
