# Error Handling

*Hush* implements two error mechanisms: *panics* and *errors*.

## Panic

A *Panic* is an **irrecoverable**[^panic] error, and will occur when there is a logic issue in your code. *Panics* will cause the whole program to crash, and execution will be terminated.

```hush
let x = 1 / 0
std.print(x)
```

Running the above script, you'll get:

```
Panic in <stdin> (line 2, column 10): division by zero
```

Examples of errors that cause a panic:
- Syntax error.
- Integer division by zero.
- Index out of bounds.
- Attempt to call a value that is not a function.
- Missing or exceeding arguments in function call.

## Error

**Recoverable** errors may be expressed through values of the *error* type. This is a special type in the language, and it is the only which cannot be expressed through a literal. Values of the error type can only be created using the `std.error` function, which expects a string description and a arbitrary context:

```hush
function check_limit(x)
	if x > 10 then
		std.error("x cannot be bigger than 10", x)
	else
		x
	end
end
```

One can then check whether the function has returned an error:

```hush
let result = check_limit(11)

if std.type(result) == "error" then
	std.print("Error: ", result)
else
	std.print("Success: ", result)
end
```

The *description* and *context* fields can be accessed as in a dictionary, but unlike in dictionaries, those cannot be assigned to:
```hush
let error = std.error("oh no!", 42)

std.print(error.description) # oh no!

error.description = "you shouldn't do that!" # panic
```

Examples of errors should be recoverable:
- File not found.
- Insufficient permission
- Invalid format
- Command not found
- Command returned non-zero exit status

## Try operator

The try (`?`[^try]) operator may be used to early return from a function if an error occurs. It is nothing but syntax sugar for an *if* expression, and therefore it may be used in any expression:


```hush
function safe_div_mod(x, y)
	if y == 0 then
		std.error("division by zero", nil)
	else
		@[ div: x / y, mod: x % y ]
	end
end

# The following are equivalent:

function foo()
	let result = safe_div_mod(5, 0)
	
	let value = if std.type(result) == "error" then
		return result
	else
		result
	end

	std.print(value)
end

function bar()
	let value = safe_division(5, 0)?
	std.print(value) # this won't be executed, as `?` will trigger an early return.
end

# The `?` operator may be used in any expression:
function run()
	std.print("div: ", safe_div_mod(5, 0)?.div)
end
```

[^panic] One can actually *catch* a *panic* using the `std.catch` function, but that should be used sparingly.

[^try] If you're familiar with both languages, *Hush*'s try operator might feel like the unholy child of Rust's `?` operator and Go's `if err != nil { return err }`.
