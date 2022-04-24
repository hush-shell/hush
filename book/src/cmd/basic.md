# Command Blocks

In *Hush*, command blocks are delimited with curly braces:

```hush
{ echo Hello world! }
```

They are valid expressions, and will result in *error* or *nil*. Therefore, we can use this result to check whether the block was successfully executed:

```hush
let result = {
	touch /etc/config # we may not have permission for that directory!
}

if std.type(result) == "error" then
	std.print("Error: ", result)
end
```

They may even be combined with the *try* operator:

```hush
function run()
	{ mkdir /etc/config/ }?
	std.print("Success!") # This won't be executed if the command block fails.
end

let result = run()
if std.type() == "error" then
	std.print("Error: ", result)
end
```

As they are blocks, they may have multiple commands, which **must** be delimited by semicolons. This enables us to include line comments in between their arguments:

```hush
{
	find . # search in the current directory.
		-type f # only files.
		-iname '*.png'; # case insensitive match on name.

	ls ../some-other-dir/images; # list files from another directory.

	cat additional-files.txt # The semicolon in the last command is optional.
}
```

## Pipelines and redirections

Pipelines and redirections use standard syntax:

```hush
{
	echo Hello world! | sed s/world/universe/g | tr '!' '.';
	echo overwrite file using stdout > file.txt;
	echo overwrite file using stderr 2> file.txt;
	echo append to file using stdout >> file.txt;
	echo stderr too 2>> file.txt;
	cat < file.txt; # redirect file to stdin
	cat << "here's an inline string"; # string to stdin
	rm file.txt 2>1; # redirect stderr to stdout. Opposed to Bash, we don't need an `&`

	# We may compose as many of those as we need:
	cat file.txt # Read a file.
			<< "Hello world!" # Concat it with data from stdin.
			2>> errors.txt # Save errors to file.
		| tee output.txt # Dump data to another file.
			2>> errors.txt # Save errors to file.
		| curl localhost:8080 -d @- # HTTP POST data to server.
			2>> errors.txt; # Save errors to file.
}
```

But there's an additional requirement for redirections: they may not precede arguments:

```hush
{ echo Hello 2>1 } # Ok.
{ echo 2>1 Hello } # Syntax error.
```

## Variables

As in most shells, *Hush* provides variable substitution in commands. But opposed to traditional shells, variables don't undergo word splitting. As *Hush* has first class support for arrays, there's really no need to do automatic word splitting. 

Variables can be used inside command blocks using dollar syntax:

```hush
let var = "hello world"
{
	echo $var; # hello world
	echo "$var"; # hello world
	echo ${var}s; # hello worlds
	echo "${var}s"; # hello worlds
	echo '$var'; # $var
}
```

*Hush* uses the following rules when doing variable substitution:

- *nil*: converted to an empty argument. Note that this is different than skipping the argument.
- *bool*, *char*, *int*, *float*, *string*: converted to string and passed as a **single** argument, regardless of containing spaces, asterisks, and whatnot.
- *array*: each element will be converted to a *single* argument, using the other rules. If the array is empty, no argument is produced. This way, arrays can be used to programmatically build lists of command arguments.
- *dict*, *function*, *error*: won't be converted, causing a *panic* instead.

Considering the file `args.hsh`:
```hush
let args = std.args() # Returns an array of command line arguments.

for arg in std.iter(args) do
	std.print(arg)
end
```

The following script
```hush
let args = [ "1 2", 3, nil, 4.0 ]

{ hush args.hsh $args }
```

will output:

```
1 2
3

4.0
```

## Expansions

In order to provide ergonomic manipulation of the file system, most shells provide a mechanism named *expansions*. It allows the programmer to refer to multiple file names using a regex-like syntax.

*Hush* provides automatic expansion **only for literal arguments**. That means you won't have to worry if your variables contains characters that may be expanded.

```hush
let var = "*"
{
	echo *; # Will print all files/directories in the current directory.
	# The following will print just an asterisk.
	echo "*";
	echo $var;
}
```

*Hush* currently provides the following expansions:

- `%`: matches zero or one character, except for the path separator.
- `*`: matches zero or more characters, except for the path separator.
- `**`: matches zero or more directories.
- `~/`: matches the `$HOME` directory, only when in the prefix of an argument.
<!-- - `[abc]`: mathes either of the characters between brackets. -->
<!-- - `{x..y}`: expands to multiple arguments, each one with a value in the range `x..y`. -->
<!-- - `{a,b,c}`: expands to multiples arguments, each one with a value from the collection `a,b,c`. -->

Opposed to traditional shells, *Hush* will *always* expand relative paths prefixed with `./`:

```hush
{
	touch test.txt; # Create a file
	echo *; # Will print "./test.txt"
}
```

You won't have to worry about flag injection from file names ever again.

## Errors

By default, whenever a command fails in a block, the whole block will be interrupted. This behavior can be disabled on a per-command basis with the `?` operator (not to be confused with the try operator outside of command blocks).

```hush
{
	echo Hello world!;

	# `false` is a command that always fails. As it's suffixed with `?`,
	# it won't cause the whole block to abort.
	false ?;

	echo "This will be printed";

	# If a command fails, and it makes no use of the `?` operator,
	# no further commands will be executed.
	false;

	echo "This will not be printed";
}
```

Command blocks will **always** result in an *error* whenever one or more of their commands fail. This is true even for commands that use the `?` operator.

```hush
let result = { false?; }
std.assert(std.type(result) == "error")
```

An error will be produced for each command that fails. This error will contain a *dict* with two fields:
- `pos`: a string describing the source position of the command.
- `status`: the numeric exit status of the program. Always non-zero.

There are scenarios where more than one command may fail, such as when using pipelines or the `?` operator. Whenever more than one command fails, the block will result in a generic error. This generic error will contain as context an array of the errors of each command that failed.

```hush
let result = { false?; false }
std.print(result.context[0])
# command returned non-zero (@[ "status": 1, "pos": "<stdin> (line 1, column 15)" ])
std.print(result.context[1])
# command returned non-zero (@[ "status": 1, "pos": "<stdin> (line 1, column 23)" ])
```
