# Builtins

Some commands must be implemented as shell builtins, so that they can mutate the shell state, a thing that would not be possible for an external program. A good example of such kind is the `cd` command, which must change the shell's working directory.

*Hush* currently provides the following bultin commands:

- `cd`: change the working directory to the given argument.

As *Hush* has no such thing as subshells, builtin commands may not be used in pipes, redirections, or capture blocks.
