# Basic Constructs

As *Hush* aims to be simple language, only basic control flow constructs and operations are supported.

## Variables

In *Hush*, all variables must be declared. This slightly increases verbosity, but makes scope rules way more reasonable, and will prevent you from ever debugging why the value of the `collcetion` variable is `nil` (note the typo).

```hush
let x # Introduces the variable in the local scope

let pi = 3.14 # Syntax sugar for assignment

let array = [ 1, 2, 3, 4 ]

let dictionary = @[
	key: pi,
	items: array,
	nested: @[ one: 1, two: 2 ],
]
```

## Operators

*Hush* provides standard arithmetic, logical, relational and indexing operators.

- **Arithmetic** `+`, `-`, `*`, `/`: *int* **or** *float*; `%`: *int* only. These operators **do not** promote *ints* to *float*, and will *panic* with mismatching types. You may explicitly convert your *ints* to *float* prior to applying the operators.
- **Logical** `and`, `or`, `not`: *bool* only. Logical operators are [short-circuiting](https://en.wikipedia.org/wiki/Short-circuit_evaluation).
- **Relational** `==`, `!=`: all types; `<`, `<=`, `>`, `>=`: *int*, *float*, *byte*, *string* only.
- **String concatenation** `++`: *string* only.
- **Indexing** `[]`, `.`: *array*, *dict*, *string*, *error* only. Attempts to access an index out of bounds will result in a *panic*. Additionally, *dicts* and *errors* may be indexed with the dot `.` operator, as long as the key is a valid identifier.

```hush
let array = [
	1 + 2,
	3.0 * std.float(4), # explicit conversion to float.
	21 % std.int(5.2), # explicit conversion to int.
	false and not true, # will short circuit.
	1 == 1.0, # false, int and float are always distinct.
	"abc" > "def",
	"hello" ++ " world",
]

std.assert(array[0] == 3)

# this would cause a panic:
# let x = array[5]

let dictionary = @[
	age: 20,
	height: 150,
	greet: function()
		std.print("hello!")
	end,
]

std.assert(dictionary["age"] == 20)
std.assert(dictionary.height == 150)
dictionary.greet() # prints "hello!"
```

As you may be wondering, the standard library, which can be used through the `std` global variable, is nothing but a dictionary full of useful functions.

## *If* expressions

In *Hush*, conditionals expect a condition of type ***bool***. Attempt to  use values of other types, such as *nil*, will result in a panic.

*If* expressions may assume two forms, with and without the `else` fragment:

```hush
let condition = true

if condition then
	# ...
end

if condition then
	# ...
else
	# ...
end
```

As they are expressions, they will evaluate to whatever is the value resulting in the last statement of the executed block. If the `else` block is ommited and the condition evaluates to `false`, the expression will result in `nil`.

```hush
let condition = false

let x = if condition then
	1
else
	2
end

std.assert(x == 2)

x = if condition then
	3
end

std.assert(x == nil)
```

## Functions

Functions are first class citizens, which means they are values like any other. The following are equivalent:

```hush
let fun = function ()
	# ...
end

function fun()
	# ...
end
```

They must declare how many arguments they expect, which is enforced when calling a function. Calling a function with less or more arguments than expected will result in a panic.

```hush
function takes_one(x)
	# ...
end

function takes_two(x, y)
	# ...
end

takes_one(1)
takes_two("a", 2)
```

Contrary to *Lua*, functions in *Hush* always return a single value, which is the result of the last statement in their body. They also may return early with the `return` keyword. The following are equivalent:

```hush
function fun(x)
	if x >= 2 then
		return # implicitly returns `nil`
	else
		return "lower than 2"
	end
end

function fun(x)
	if x < 2 then
		"lower than 2"
	end
end
```

*Hush* implements [lexical scoping](https://en.wikipedia.org/wiki/Scope_(computer_science)#Lexical_scope_vs._dynamic_scope_2), which means variables are enclosed in the body in which they are declared, just like in *Python* and *Lua*. It also supports closures, which are functions that capture variables from the enclosing scope:

```hush
function adder(x)
	let y = x + 1

	return function (z) # `return` may be ommited here
		y + z # captures `y` from the parent scope.
	end
end

std.assert(adder(1)(2) == 4)
```

Closures may even mutate the captured varibles:

```hush
let x = 0

function increment()
	x = x + 1
end

increment()
increment()
increment()

std.assert(x == 3)
```

Functions can also be recursive. As they are values, recursive functions are actually closures on themselves (they capture the variable to which they are assigned).

```hush
function factorial(n)
	if n == 0 then
		1
	else
		n * factorial(n - 1)
	end
end

std.assert(factorial(5) == 120)
```

## *While* loops

*While* loops are statements, and therefore cannot be used as expressions.

```hush
let condition = true
let i = 0

while condition do
	condition = false
	i = i + 1
end

std.assert(i == 1)
```

## *For* loops

*For* loops are also statements, but opposed to *While* loops, they do not expect a boolean condition. First, they expect a variable name, which will be scoped to the loop's body. Second, they expect an iterator function.

An iterator function is a function that may be called repeatedly without arguments, and always returns a dictionary with at least one field:
- `finished`: a boolean indicating whether the loop should stop.
- `value`: the value to be assigned to the loop variable. May be ommited if `finished` is `false`.

```hush
# A function to generate an iterator to the given array.
function iter(array)
	let i = -1
	let len = std.len(array)

	function ()
		i = i + 1 # captures `i`, incrementing it on every call.

		if i == len then # check if we reached the captured `len`.
			@[ finished: true ]
		else
			@[ finished: false, value: array[i] ]
		end
	end
end

let array = [1, 2, 3]
let sum = 0

for item in iter(array) do
	sum = sum + item
end

std.assert(sum == 6)
```

Fortunately, the `iter` function defined above is present in the standard library, as `std.iter(collection)`. For numeric iterations, the standard library also supplies the `std.range(from, to, step)` function, which returns an iterator:

```hush
let sum = 0

for i in std.range(1, 4, 1) do
	sum = sum + i
end

std.assert(sum == 6)
```

## *Break* statement

One may also interrupt loops using the `break` statement:

```hush
while true do # this will not run forever.
	if 1 + 2 < 4 then
		break
	end
end
```

## Wrapping up

With these constructs, you should be able to write basic programs in *Hush*. Next, we'll learn how to implement proper error handling, as robustness is one of the core values of the language.
