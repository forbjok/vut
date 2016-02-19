# Vut [![Build Status](https://travis-ci.org/forbjok/vut.svg?branch=master)](https://travis-ci.org/forbjok/vut)

## Introduction
Vut is a versioning utility.
It lets you easily keep track of a project's version and do things like bump its individual parts, including prerelease or build strings that end in numbers.
Its main strength, however is in the way it propagates version numbers to where they are needed - using templates.

Ideally, it would be sufficient to store the one true version number in only a single place, and every other place that requires it would just read it from that one place.
Unfortunately, realistically that's just not doable most of the time.
Rather than rely on inflexible and specific code for generating files containing the version in specific formats or languages, or having to write equally specific scripts, Vut allows propagation of versions by using templates.

## Installing
For Windows, there is a [Chocolatey](https://chocolatey.org/) [package](https://chocolatey.org/packages/vut) available.
```
C:\> choco install vut
```

For GNU/Linux and other operating systems, you will have to compile it yourself.

## Compiling
1. Download and install the lastest version of DMD (the reference D compiler) from [dlang.org](http://dlang.org/) or your distro's package manager
2. Download and install DUB from [code.dlang.org](https://code.dlang.org/) or your distro's package manager
3. Clone this repository and execute the following command in it:
```
$ dub build
```

Voila! You should now have a usable executable in the root of the repository.

## Creating a VERSION file
In order to store the ONE TRUE VERSION, Vut uses a file called VERSION.
It's a plain UTF-8 encoded text file containing the full SemVer string, with no newline at the end.

The recommended way of creating one is to run the following command in the root of your repository (or wherever the root of the versioned content is):
```
$ vut init 1.0.0
```
... where 1.0.0 can be substituted with any SemVer 2.0 compatible version string that will be used as the initial version.

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
Simply write a template file manually, in whatever language or format you need it to be in and place it anywhere within your project structure naming it whatever you need the generated file to be called with the extension .vutemplate appended to the end.

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


## Setting the version
The version number can be set at any time by using `vut set` like this:
```
$ vut set 1.0.2-beta.3+build42
```

## Getting the version number
Sometimes you may want to easily get the current version, or some component of it - for example in a build script.
That's the purpose `vut get` is designed for.
```
$ vut get
```
will output all the available variables in json format:
```json
{
    "build": "build42",
    "buildNumber": "42",
    "buildPrefix": "build",
    "fullVersion": "1.0.2-beta.3+build42",
    "major": "1",
    "majorMinor": "1.0",
    "majorMinorPatch": "1.0.2",
    "minor": "0",
    "patch": "2",
    "prerelease": "beta.3",
    "prereleaseNumber": "3",
    "prereleasePrefix": "beta.",
    "version": "1.0.2-beta.3"
}
```

If you need to retrieve a custom version string, whether it's just a single variable or a combination of many (for example for use in a shell script), you can use:
```
$ vut get --format="{{MajorMinorPatch}} {{PrereleasePrefix}}{{PrereleaseNumber}}"
```
to get output like `1.0.2 beta.3`.
The format here is exactly the same as in templates. Anything you can put in a template, you can put in a --format string and vice versa.

*Note that all the variables start with a lowercase letter in the json output, whereas they must start with a capital letter when used in templates or format strings.*

## Regenerating templates without changing the version
Sometimes you may want to regenerate all templates even though the version hasn't changed. For example, if you've changed or added a template.
You can do that by using:
```
$ vut generate
```
... or `vut gen` for short.

## Custom prefixes and suffixes
In addition to simply substituting the version number, you can specify a prefix or suffix for any variable by placing the prefix or suffix between two pipe symbols before or after the variable name like `{{|prefix-|Prerelease}}`.

A variable's prefix or suffix will only be inserted if the variable itself is non-empty.

A prefix or suffix can be any string of characters except a pipe symbol, for obvious reasons.
