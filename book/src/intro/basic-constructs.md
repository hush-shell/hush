# Basic Constructs

As *Hush* aims to be simple language, only standard control flow constructs and operations are supported.

## Comments

Comments start with a `#`:

```hush
# This is a comment.
```

## Variables

In *Hush*, all variables must be declared. This slightly increases verbosity, but makes scope rules way more reasonable, and will prevent you from ever debugging why the value of the `collcetion` variable is `nil` (note the typo).

```hush
let x # Introduces the variable in the local scope

let pi = 3.14 # Syntax sugar for assignment

let string = "hello!"

let byte = 'a', # this is a single byte, not a string. Note the single quotes.

let array = [ 1, 2, 3, 4 ]

let dictionary = @[
	key: pi,
	items: array,
	nested: @[ one: 1, two: 2 ],
]
```

Identifiers must start with a letter or underscore, and may contain further letters, digits and underscores.

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
