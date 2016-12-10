module utils.indentation;

import std.regex : ctRegex, replaceAll;

auto ctrLeadingTabs = ctRegex!(`(?<!^)\t`);

string spacify(in string s) {
    /* Replace all tabs with 2 spaces. */
    return replaceAll(s, ctrLeadingTabs, "  ");
}
