# Installation

*Hush* comes as a single binary, which can be installed on Unix systems without much hassle.

## Packaged distributions

Currently, *Hush* is packaged for the following operating systems:

- Arch Linux: [AUR](https://aur.archlinux.org/packages/hush-bin)

## Binary download

Precompiled binaries may be downloaded from the [Github releases](https://github.com/hush-shell/hush/releases) page. Currently, there is only a x86 Linux binary. If you would like to help by providing binaries for other architectures or operating systems, please let me know.

## Building from source

*Hush* can be installed from source using [cargo](https://crates.io/crates/hush):

```bash
cargo install hush
```

## Editor support

The following editors have plugins available:

- Emacs: [hush-mode.el](https://github.com/hush-shell/hush/blob/master/syntax-highlight/emacs/hush-mode.el). I'll release it on Melpa as soon as I find the time.
- VsCode: [marketplace](https://marketplace.visualstudio.com/items?itemName=hush-vscode.hush)
