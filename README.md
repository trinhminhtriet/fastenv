# ⚙️ FastEnv

```text
  __              _
 / _|  __ _  ___ | |_   ___  _ __  __   __
| |_  / _` |/ __|| __| / _ \| '_ \ \ \ / /
|  _|| (_| |\__ \| |_ |  __/| | | | \ V /
|_|   \__,_||___/ \__| \___||_| |_|  \_/
```

⚙️ FastEnv: Unintrusive, on-demand environment manager creating lightweight shims for projects, without shell hooks or auto-loading scripts.

## ✨ Features

- **On-Demand Environment Management**: Unlike traditional tools, `fastenv` doesn’t automatically hook into your shell or execute scripts on directory changes. This keeps your shell clean and responsive, activating environments only when explicitly needed.

- **Manual Reloading for Flexibility**: With `fastenv`, environment variables are loaded via `fastenv reload`, giving you full control over when changes take effect. Simply run this command whenever you update your `.envrc`, ensuring no unintended behavior or lag from automatic loading.

- **Shim-Based Execution**: Instead of modifying your shell environment directly, `fastenv` uses shims—small binaries that point to the required executables. This means commands within each project’s environment are executed seamlessly, without polluting your global environment.

- **Seamless Compatibility with Existing `.envrc` Files**: `fastenv` is fully compatible with existing `.envrc` files, making it easy to adopt without rewriting environment scripts. Drop in `fastenv`, and your environment configurations work as expected.

- **Inspired by Volta’s Efficient Node.js Management**: Borrowing ideas from Volta’s approach to Node.js version management, `fastenv` provides targeted control over environments while avoiding interference with your shell setup.

## 🚀 Installation

**fastenv is work in progress. that said, I use it daily at work**

[Download the latest binary](https://github.com/trinhminhtriet/fastenv/releases) and:

```bash
# Into your bashrc/zshrc. This should be at the front of your PATH, such that
# fastenv can shim/shadow binaries effectively.
export PATH=$HOME/.fastenv/bin/:$PATH

# You can remove "direnv hook" from your bashrc/zshrc, but the tool needs to
# stay installed.
```

Some notes:

- `fastenv` currently assumes `direnv` is in your path, in order to load its
  "standard library".

- `fastenv` also currently does not have pre-built binaries. You need to
  [install Rust](https://rustup.rs/) and install it using Rust's package
  manager, Cargo.

- `fastenv` assumes a POSIX environment.

### Installation from source

```bash
cargo install fastenv  # latest stable release
cargo install --git https://github.com/trinhminhtriet/fastenv  # latest git SHA
```

## 💡 Usage

````bash
# Alternatively you can shim commands explicitly. Be careful: Any command you
# missed (such as 'python' or 'pip') would run outside of the virtualenv!
fastenv shim sentry pytest

# You can also run commands within the current .envrc without shimming them.
fastenv exec -- pytest

# Your git hooks don't execute in the virtualenv for some reason? Just replace
# git with a binary that itself loads the virtualenv.
fastenv shim git

# Actually activate the virtualenv in your current shell. `fastenv vars`
# prints all the extra environment variables with which each shimmed binary runs.
set -o allexport
eval "$(fastenv vars)"
set +o allexport

# Or alternatively, substitute your shell with one where your .envrc is loaded
exec fastenv exec $SHELL

# Or shim 'bash', so that when you open a subshell, the virtualenv is activated.
fastenv shim bash

# Or shim 'make', so your Makefile runs in the virtualenv.
fastenv shim make

# Curious which binary is actually being executed?
fastenv which make
# /home/user/.fastenv/bin/make

# Or for general debugging, increase the log level:
QUICKENV_LOG=debug make
# [DEBUG fastenv] argv[0] is "make"
# [DEBUG fastenv] attempting to launch shim
# [DEBUG fastenv] abspath of self is /home/user/.fastenv/bin/make
# [DEBUG fastenv] removing own entry from PATH: /home/user/.fastenv/bin
# [DEBUG fastenv] execvp /usr/bin/make
# ...
```

## 🗑️ Uninstallation

Running the below command will globally uninstall the `fastenv` binary.

```bash
cargo uninstall fastenv
````

Remove the project repo

```bash
rm -rf /path/to/git/clone/fastenv
```

## 🤝 How to contribute

We welcome contributions!

- Fork this repository;
- Create a branch with your feature: `git checkout -b my-feature`;
- Commit your changes: `git commit -m "feat: my new feature"`;
- Push to your branch: `git push origin my-feature`.

Once your pull request has been merged, you can delete your branch.

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
