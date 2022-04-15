# Command Blocks

Command blocks is the feature that distinguishes *Hush* from ordinary programming languages. They allow *Hush* scripts to seamlessly invoke and interconnect external programs.

Great effort has been put to make command syntax in *Hush* as similar as we're used to in *Bash*, but some key aspects have been changed in order to favor robustness of scripts. You should never have to use something like the [unnoficial bash strict mode](http://redsymbol.net/articles/unofficial-bash-strict-mode/) in *Hush*:

```bash
#!/bin/bash
set -euo pipefail
IFS=$'\n\t'
```
<p align="center">
	<small>The unnoficial bash strict mode</small>
</p>

Let's see why you won't need any of this in *Hush*:
- `set -e`: interrupt execution immediately if a command has non-zero exit status. This is the default behavior for command blocks in *Hush*.
- `set -u`: exit with an error on any attempt to use an undeclared variable. *Hush* won't even start to execute your script if you mention an undeclared variable.
- `set -o pipefail`: if a command in a pipeline fails, make the whole pipeline fail. This is the default behavior in *Hush*, and can even be controlled on a per-command basis in pipelines.
- `IFS=$'\n\t'`: do word-splitting using only newlines and hard tabs. *Hush* does no word splitting whatsoever, so this will never be a source of confusion or bugs.

In the next sections, we'll learn how to use command blocks in *Hush*.
