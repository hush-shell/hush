# Type System

*Hush* is strongly dynamically typed, which means all **values** have a **well formed type**, but **variables** are **untyped**. Therefore, you may assign values of distinct types to a given variable.

As in *Lua*, *Hush* proposes only a handful of built-in types, and no user-defined types. This makes the type system extremely simple, and yet still it remains impressively expressive. The following types are available:

- `nil`: the unit type, usually for representing missing values.
- `bool`: the boolean type.
- `int`: a 64 bit integer type.
- `float`: a 64 bit floating point type.
- `char`: a C-like unsigned char type, 0-255.
- `string`: a char-array like string.
- `array`: a heterogeneous array, 0-indexed (unlike in *Lua*).
- `dict`: a heterogeneous hash map.
- `function`: a callable function.
- `error`: a special error type, to ease distinction of errors from other values. This type can only be instantiated by the built-in `std.error` function.

Although it may seem like a limitation to only have a handful of types, *Hush* provides facilities that enable these types to be extremely flexible. We'll get more in depth about that in practice on the [Paradigms](../paradigms/index.md) section.
