# Standard Library

*Hush*'s standard library is available through the `std` global variable, which is implicitly available to all scripts. It is nothing but a dictionary of predefined functions, which can be used like any other user defined functions.

Currently, the following functions are available:

#### `std.args()`
Gets an array of command line arguments.

#### `std.assert(value)`
Asserts that `value` is `true`, panics otherwise.

#### `std.bind(obj, function)`
Returns a new function that binds `function` to `obj`, so that `self` will always refer to `obj` when the function is called.

#### `std.catch(function)`
Runs the given `function`, and returns an error if it panics. This should be used sparingly, as panics shouldn't be used for recoverable errors.

#### `std.cd(dir)`
Change the current directory to `dir`.

#### `std.contains(collection, value)`
Checks if `collection` contains an item that is equal to `value`. May be used with *string*s, *array*s and *dict*s.

#### `std.cwd()`
Returns the current working directory.

#### `std.env(key)`
Gets the value of the environment variable `key`, or *nil* if it's not defined.

#### `std.error(description, context)`
Create a new *error* with the given description and context. The description must be a string.

#### `std.export(key, value)`
Set the environment variable `key` to `value`. Both arguments must be *string*s.

#### `std.float(value)`
Convert `value` to *float*. Accepts *string*, *int* and *float*.

#### `std.glob(path)`
Expands the given `path` in the current directory, using the shell expansion rules (`*`, `%`, etc).

#### `std.has_error(value)`
Recursively checks if `value` contains a value of type *error*.

#### `std.import(path)`
Load the *Hush* script from the given path, relative to the current file.

#### `std.int(value)`
Convert `value` to *int*. Accepts *string*, *int* and *float*.

#### `std.is_empty(collection)`
Checks if the given `collection` is empty. Accepts *string*s, *array*s and *dict*s.

#### `std.iter(collection)`
Returns an iterator function for the given `collection`. Accepts *string*s, *array*s and *dict*s.

#### `std.json.dump(value)`
Convert the given value to a JSON *string*. Panics if `value` contains a value that cannot be serialized as JSON (*function* or *error*).

#### `std.json.parse(string)`
Parse the given json `string`. Returns an *error* if parsing fails.

#### `std.length(collection)`
Returns the amount of elements in the given collection. Accepts *string*s, *array*s and *dict*s.

#### `std.panic(value)`
Panics with the given `value` as description.

#### `std.pop(array)`
Removes the last element from the given `array`, returning it. Panics if the array is empty.

#### `std.print(value)`
Prints the given value to *stdout*, including a newline.

#### `std.push(array, value)`
Adds the given `value` to the end of `array`.

#### `std.range(from, to, step)`
Returns an iterator function that yields numbers in the given range.

#### `std.read(prompt)`
Read a line from *stdin*, using the given `prompt`, which must be a *string*.

#### `std.replace(string, seach, replace)`
Replace occurrences of `search` with `replace` in `string`. All parameters must be *string*s.

#### `std.sleep(ms)`
Sleep for the given amount of milliseconds. Accepts positive integers only.

#### `std.sort(array)`
Sorts the given array.

#### `std.split(string, pattern)`
Splits the given `string` by occurrences of `pattern`, returning a non-empty array. Both arguments must be *string*s.

#### `std.substr(string, from, length)`
Slice the given `string`. The two last parameters must be positive integers.

#### `std.to_string(value)`
Converts the given value to *string*.

#### `std.trim(string)`
Removes whitespace from the start and end of the given *string*.

#### `std.typecheck(value, type)`
Checks if the given `value` has type `type`, panics otherwise. `type` must be a *string*.

#### `std.try_typecheck(value, type)`
Checks if the given `value` has type `type`, returns an *error* otherwise. `type` must be a *string*.

#### `std.type(value)`
Returns a *string* describing the type of `value`: `"nil"`, `"bool"`, `"char"`, `"int"`, `"float"`, `"string"`, `"array"`, `"dict"`, `"function"`, or `"error"`.
