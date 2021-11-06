Vut
===

[![CI](https://github.com/forbjok/vut/actions/workflows/ci.yml/badge.svg)](https://github.com/forbjok/vut/actions/workflows/ci.yml)
![GitHub release (latest by date)](https://img.shields.io/github/v/release/forbjok/vut)
![Chocolatey Version](https://img.shields.io/chocolatey/v/vut)

## Introduction

Vut is a utility for propagating version numbers.

**Scenario:** You have a repository containing a number of projects and files that are part of a single unit of distribution. Maybe there's a C project in which you want to access the version from code to display it to the user. Maybe there's a NuGet or Chocolatey .nuspec file. Maybe there's a web front-end that uses NPM.

You want everything in this repository to have the same version number. But... in order to do this, you need to update it in multiple files every time you change it. And then one time you forget, and suddenly the version is wrong in some obscure place.

This is a pain. And it's where Vut comes in.

By setting the version using Vut, the process of propagating it to every other file in which it is needed can be completely automated.

It has a few different mechanisms for accomplishing this:
* Templates -- Generate files containing version numbers from a template.
* Version sources -- Update inner version sources to match the authoritative version source.
* File updaters -- Replace version numbers within existing files using *regular expressions* to find and replace them.

## Installing
For Windows, there is a [Chocolatey](https://chocolatey.org/) [package](https://chocolatey.org/packages/vut) available.
```
C:\> choco install vut
```

For GNU/Linux and other operating systems, you will have to compile it yourself.

## Compiling
1. Install Rust using the instructions [here](https://www.rust-lang.org/tools/install) or your distro's package manager.
2. Clone this repository and execute the following command in it:
```
$ cargo build --release
```

Voila! You should now have a usable executable in the `target/release` subdirectory.

## Configuration

While it is possible (for backward-compatibility reasons) to use Vut without a configuration file, it is recommended to have one, and it is required in order to take advantage of the more advanced functionality, such as updating version sources or updating files.

A basic minimal configuration file can be created by running:
```
$ vut init
```
in the root directory of the project. If no version source is present, this will create a `vut.toml` file and a `VERSION` file. If a version source is found, only a `vut.toml` file will be created and the existing version source will be used.

For a detailed overview and explanation of all existing configuration options, an example configuration can be created by running:
```
$ vut init --example
```
This configuration file contains examples and explanatory comments for all existing configuration options.

## Authoritative Version Source
In order to do its work, Vut requires an authoritative version source, which will typically be located in the same directory as the configuration file.

The authoritative version source can be of any supported version source type.

## Version sources

Version sources are systems that support getting and setting a version number to a particular file format. These are primarily used for the **authoritative version source**, but can also be used to update additional (non-authoritative) version sources within the project directory.

The currently supported built-in version sources are: **vut**, **cargo** and **npm**.

### **vut** -- Vut VERSION file
The default Vut version source.
It's a plain UTF-8 encoded text file called `VERSION`, containing the full SemVer string, with no newline at the end.
This should generally be used for any project type that doesn't have its own package manifest containing a version.

### **cargo** -- Cargo.toml (Rust)
Cargo.toml is the package manifesto file used by Rust's package manager Cargo.

### **npm** -- package.json (NPM)
package.json is the package description format used by the NPM package manager.

## Bumping a version
To bump a version component, use any one of:
```
$ vut bump major
$ vut bump minor
$ vut bump patch
$ vut bump prerelease
$ vut bump build
```
... depending on which version component you want to increase.

Bumping any version component will increase it by one and cause all lesser ones to be reset to 0, or in the case of prerelease or build, removed.
Bumping prerelease or build requires that component to be present and end in a number.

## Using templates
Simply write a template file manually, in whatever language or format you need it to be in and place it anywhere within your project structure naming it whatever you need the generated file to be called with the extension .vutemplate (by default - this is configurable) appended to the end.

For example, if you need the version number to be directly accessible in a C# program, you can create a file called **AppVersion.cs.vutemplate** somewhere in your project structure.
```C#
public static class AppVersion
{
	public const string Version = "{{Version}}";
}
```

The next time the version is changed throught Vut, a file called **AppVersion.cs** will be created in the same location as the template, with {{Version}} replaced with the actual version, which you can then add to your project and use in your code.
There are a number of variables available other than {{Version}} as well, allowing you to construct version strings in just about any way.

## How templates work
Any time a version change is performed through Vut, or `vut generate` is run, Vut will search the entire directory structure for **.vutemplate**s, starting from the location of the VERSION file and generate files of the same name minus the .vutemplate extension in the same directory as the template with any version variables replaced.

Just like with most revision control systems, discovery of the VERSION file for the current working directory works by traversing the current path outward until a directory containing a VERSION file is found, meaning you can run Vut from anywhere within the versioned project structure.

## File updaters

In cases where it is not possible or desirable to use templates, you can use a file updater to replace a version number in any text file as long as it is possible to locate using a regular expression.

Each file updater can contain multiple replacers, each with their own regexes and template string. By default, if no template string is specified, the full version string (`{{FullVersion}}`) will be used.

Example configuration:

```toml
# Define a custom file updater.
[file-updaters.myfile]
type = "regex"
replacers = [
  { regexes = ["(^Version = )(.*)(;)", "(^FullVersion = )(.*)(;)"] },
  { regexes = "(^ShortVersion = )(.*)(;)", template = '{{MajorMinor}}' },
]

# Update files using a file updater.
[[update-files]]
globs = "**/*.myfile"
updater = "myfile"
encoding = "utf-8"
```

## Setting the version
The version number can be set at any time by using `vut set` like this:
```
$ vut set 1.0.2-beta.3+build42
```

## Getting the version number
Sometimes you may want to easily get the current version, or some component of it - for example in a build script.
That's the purpose `vut get` is designed for.
```
$ vut get json
```
will output all the available variables in json format:
```json
{
  "Build": "",
  "BuildNumber": "",
  "BuildPrefix": "",
  "FullVersion": "0.1.0",
  "Major": "0",
  "MajorMinor": "0.1",
  "MajorMinorPatch": "0.1.0",
  "Minor": "1",
  "Patch": "0",
  "Prerelease": "",
  "PrereleaseNumber": "",
  "PrereleasePrefix": "",
  "Version": "0.1.0"
}
```

## Re-propagating versions without changing the version
Sometimes you may want to re-propagate versions and regenerate all templates even though the version hasn't changed. For example, if you've changed or added a template.

You can do that by using:
```
$ vut generate
```
... or `vut gen` for short.

## Custom prefixes and suffixes
In addition to simply substituting the version number, you can specify a prefix or suffix for any variable by placing the prefix or suffix between two pipe symbols before or after the variable name like `{{|prefix-|Prerelease}}`.

A variable's prefix or suffix will only be inserted if the variable itself is non-empty.

A prefix or suffix can be any string of characters except a pipe symbol, for obvious reasons.
