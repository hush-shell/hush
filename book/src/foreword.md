# Foreword

This is the official guide of *Hush*, a modern shell scripting language. A shell scripting language is a domain specific language which provides constructs for easily invoking and interconnecting external programs. These kind of languages are typically used for integrations in Unix systems and infrastructure programming. [GNU Bash](https://www.gnu.org/software/bash/), [Zsh](https://www.zsh.org/) and [Fish](https://fishshell.com/) are some of the most commonly used ones.

## But why do we need a new shell scripting language?

Traditional shell scripting languages are notoriously error prone and frequently result in programs that are hard to maintain. Even Google mentions it on their [Shell Style Guide](https://google.github.io/styleguide/shellguide.html#when-to-use-shell):

> If you are writing a script that is more than 100 lines long, or that uses non-straightforward control flow logic, you should rewrite it in a more structured language now. Bear in mind that scripts grow. Rewrite your script early to avoid a more time-consuming rewrite at a later date.

Hush strives to enable the development of robust and maintainable shell scripts. It does so by providing industry proved programming constructs, support for beyond trivial data structures, sane error handling and variable expansion. While most shells are command interpreters that strive to become programming languages, Hush is more like a general purpose programming language with added first class shell capabilities.
