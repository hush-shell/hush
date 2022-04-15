# Introduction

In this section, we'll learn more about the basic constructs provided by *Hush*. That'll enable you to implement *Hush* scripts using functions, data structures, and control flow.

## Hello World

With *Hush* installed, we're able to execute scripts. Here's the traditional *Hello world* program in *Hush*:

```hush
#!/usr/bin/env hush

std.print("Hello world!")
```

The first line is a [Shebang](https://en.wikipedia.org/wiki/Shebang_(Unix)). It tells the operating system which interpreter to use for the script. The second line is a function call of the `print` function from the standard library. When executed, this script will output `Hello world!` to the standard output.

## Making the script executable

To execute this script, save it to a file named `hello-world.hsh`, give it execution permission, and then run it:
```shell
$ chmod +x hello-world.hsh
$ ./hello-world.hsh
Hello world!
```

## Calling Hush directly

You can also execute a given script calling *Hush* with the script path as argument. When executing a script using this method, the *Shebang* is unnecessary, and will be disregarded as an ordinary comment.
```shell
$ hush hello-world.hsh
Hello world!
```

## Tooling

*Hush* provides some tools for static analyses of scripts, which can be invoked by passing flags to the shell. The most useful one is the `--check` flag, which will check the script for syntax and semantic errors, without executing it.

Consider the following `script.hsh`, which attempts to use the undeclared variable `value`:

```hush
value = "Hello world!"
std.print(value)
```

We can check verify that the script has semantic errors:

```shell
$ hush --check script.hsh
Error: script.hsh (line 1, column 0) - undeclared variable 'value'
Error: script.hsh (line 2, column 10) - undeclared variable 'value'
```

To get a list of other useful flags, run `hush --help`.
