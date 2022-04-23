# Async

*Hush* also provides a way to launch commands asynchronously, and then wait for their result at a later time. Traditional shells provide similar functionaly through the ampersand operator.

To run a command block asynchronously, prefix it with an ampersand:

```hush
let handle = &{ echo Hello world! }

# Do some other work before calling join.
# This may be printed before or after "Hello world!".
std.print("Doing some work...")

# This will wait until the block runs to completion, and will return it's result.
let result = handle.join()

std.assert(result == nil)
```

Async blocks will start executing immediately, but *Hush* won't wait for their completion until *join* is called, and will continue script execution instead. An async block will **always** result in a *dict* containing the *join* method, which will then return the block result (*nil* or *error*) when called.
