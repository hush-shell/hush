# Capture

One of the most important features of a shell is to be able to manipulate the standard I/O streams of programs. There are three main ways of doing so: pipes, redirection, and capturing. The first two are commonly used when we just want to forward the output or input of a program. But if we want to preprocess the output, or use it as an argument to another program, we'll usually reach for capturing the output stream as a string in the shell, so that we can handle it ourselves.

In *Hush*, we can capture the output of a whole command block by prefixing it with a dollar:

```hush
let result = ${ echo Hello world! }
```

Instead of resulting in *nil* or *error*, as does the standard command block, a capture block will result in either a *dict* containing the *stdout* and *stderr* string fields, or an *error* if the block fails.

```hush
function get_first_word()
	let output = ${ echo Hello world! }?.stdout
	let first_word = std.split(std.trim(output), " ")[0]
	if std.is_empty(first_word) then
		std.error("output was empty", output)
	else
		first_word
	end
end

std.print(get_first_word()) # Hello
```

The separation of the *stdout* and *stderr* fields in the resulting *dict* enables the programmer to properly handle the standard error stream separately from the standard output stream, something that is more complicated than it should in traditional shells.

If the command block fails, the resulting *error* will contain the output that captured was captured before the block failed:

```hush
let result = ${
	echo Hello world!;
	echo The next command wil fail 1>2;
	this-command-does-not-exists;
}

std.assert(std.type(result) == "error")
std.assert(result.context.stdout == "Hello world!\n")
std.assert(result.context.stderr == "The next command wil fail\n")
```
