module utils.filelocator;

import std.algorithm;
import std.file;
import std.path;

string locateFileInPathOrParent(in string startPath, in string filename) {
    auto path = startPath.absolutePath();
    assert(path.isDir());

    auto prevPath = "";
    while(path != prevPath) {
        // Construct the full path to the desired file in the current path
        auto fullPath = buildPath(path, filename);

        // Check if the file exists
        if (fullPath.exists() && fullPath.isFile()) {
            // File existed, so return its path
            return fullPath;
        }

        // Store previous path
        prevPath = path;

        // Set path to previous path's parent directory
        path = path.dirName();
    }

    return null;
}

auto locateTemplates(in string startPath, in string templateExtension) {
    auto files = dirEntries(startPath, SpanMode.breadth, false)
        .filter!(f => f.name.endsWith(templateExtension));

    return files;
}
