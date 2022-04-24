# Control Flow and Functions

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

As they are expressions, they will evaluate to whatever is the value resulting in the last statement of the executed block. If the `else` block is omitted and the condition evaluates to `false`, the expression will result in `nil`.

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

Closures may even mutate the captured variables:

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

### Self

*Hush* provides one further facility for functions: the `self` keyword. When calling a function inside a dictionary using the dot operator, `self` will be an alias to that dictionary. If the function is called through other means, `self` will be `nil`. This is frequently used in [object oriented](../paradigms/oop.md) code.

```hush
let dict = @[
	value: 5,

	method: function()
		# `self` is a reference to the dictionary which contains the function, if any.
		std.print(self)
	end
]

dict.method() # @[ "value": 5, "method": function<...> ]

# Isolate the method from the object, which will cause `self` to be `nil`:
let method = dict.method
method() # nil

# But we can bind it back to the object using `std.bind(obj, method)`:
method = std.bind(dict, dict.method)
method() # @[ "value": 5, "method": function<...> ]
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
- `value`: the value to be assigned to the loop variable. May be omitted if `finished` is `false`.

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
