# `jvr`

A simple and easy-to-use Java version manager(registry: `jvr`), similar to `Node.js`'s `nvm`,
but it does not follow `nvm`'s naming convention. Otherwise, it would be named `jvm`,
which could cause command conflicts or ambiguity.

## 0.`Platform Support`

`jvr` supports the following operating systems:

- ✅ **Windows** (Windows 10/11)
- ✅ **Linux** (various distributions)
- ✅ **macOS** (various versions)

## 1.`Install`

### 1.1.`Download executable `

Download the executable file directly from `GitHub` and put it in any `$PATH` path.

### 1.2.`Cargo`

```shell
cargo install --git https://github.com/photowey/jvr.git [--branch main]
```

### 1.3.`Build from source`

```shell
git clone https://github.com/photowey/jvr.git
cd jvr
cargo build --release
```



## 2.`Usage`

### 2.1.`Commands`

#### 2.1.1`Add`

Register `JDK`

```shell
$ jvr add -h | --help
$ jvr add <NAME> <PATH>

# e.g.:
$ jvr add jdk8 ${YOUR_PATH}/jdk8
$ jvr add jdk11 ${YOUR_PATH}/jdk11
$ ...
```

#### 2.1.2`List`

List all registered `JDK`s, and use a clear table to list the registrations. The `*` indicates the version currently in use.

```shell
$ jvr list
```

#### 2.1.3`Use`

Switch the `JDK` version and automatically update the user's `JAVA_HOME` environment variable.

```shell
$ jvr use <NAME>

# e.g.:
$ jvr use jdk11
```

**Note for Unix users (Linux/macOS):**

After using `jvr use`, the `JAVA_HOME` environment variable is written to your shell configuration file
(e.g., `~/.bashrc`, `~/.zshrc`, `~/.profile`). To apply the changes, you need to:

1. **Reload your shell configuration:**
   ```shell
   # For bash
   source ~/.bashrc
   # or
   source ~/.bash_profile
   
   # For zsh
   source ~/.zshrc
   
   # For profile
   source ~/.profile
   ```

2. **Or simply open a new terminal window/tab**

The changes will take effect immediately in Windows.

#### 2.1.4`Version`

View the version of `jvr` itself.

#### 2.1.5`Open`

Open the directory where `jvr` `HOME` is located, if applicable.

## 3.`Configuration`

### 3.1.`Configuration File Location`

`jvr` stores its configuration in:

- **Windows**: `%USERPROFILE%\.jvr\config.json`
- **Linux/macOS**: `~/.jvr/config.json`

### 3.2.`Environment Variables`

`jvr` automatically manages the following environment variables:

- **JAVA_HOME**: Set to the currently active JDK path
- **PATH**: Automatically includes `$JAVA_HOME/bin` (Unix) or `%JAVA_HOME%\bin` (Windows)

### 3.3.`Shell Configuration Files (Unix)`

On Unix systems (Linux/macOS), `jvr` automatically detects your shell and writes environment variables to the appropriate configuration file:

- **Zsh**: `~/.zshrc`, `~/.zshenv`
- **Bash**: `~/.bashrc`, `~/.bash_profile`, `~/.profile`
- **Default**: `~/.profile`

The tool will use the first available file in the list above.

## 4.`Examples`

### 4.1.`Basic Usage`

```shell
# Add JDK versions
$ jvr add jdk8 /usr/lib/jvm/java-8-openjdk
$ jvr add jdk11 /usr/lib/jvm/java-11-openjdk
$ jvr add jdk17 /usr/lib/jvm/java-17-openjdk

# List all registered JDKs
$ jvr list

# Switch to a specific JDK version
$ jvr use jdk17

# Verify the change (Unix)
$ echo $JAVA_HOME
$ java -version

# Verify the change (Windows)
$ echo %JAVA_HOME%
$ java -version
```

### 4.2.`Platform-Specific Examples`

**Windows:**
```shell
$ jvr add jdk21 "C:\Program Files\Java\jdk-21"
$ jvr use jdk21
```

**Linux/macOS:**
```shell
$ jvr add jdk21 /usr/lib/jvm/java-21-openjdk
$ jvr use jdk21
$ source ~/.bashrc  # or ~/.zshrc
```

## 5.`Troubleshooting`

### 5.1.`Environment Variable Not Updated (Unix)`

If `JAVA_HOME` is not updated after using `jvr use`:

1. Check if the environment variable was written to your shell config file:
   ```shell
   $ cat ~/.bashrc | grep JAVA_HOME
   # or
   $ cat ~/.zshrc | grep JAVA_HOME
   ```

2. Reload your shell configuration:
   ```shell
   $ source ~/.bashrc  # or ~/.zshrc
   ```

3. Verify in a new terminal window.

### 5.2.`Permission Denied (Unix)`

If you encounter permission errors when setting system environment variables, you may need to use `sudo`:

```shell
$ sudo jvr use <NAME>
```

However, it's recommended to use user-level environment variables instead.

## 6.`License`

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.