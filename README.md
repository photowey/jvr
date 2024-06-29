# `jvr`

A simple and easy-to-use Java version manager(registry: `jvr`), similar to `Node.js`'s `nvm`,
but it does not follow `nvm`'s naming convention. Otherwise, it would be named `jvm`,
which could cause command conflicts or ambiguity.

## 1.`Install`

### 1.1.`Download executable `

Download the executable file directly from `GitHub` and put it in any `$PATH` path.

### 1.2.`Cargo`

```shell
cargo install --git https://github.com/photowey/jvr.git [--branch main]
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

#### 2.1.4`Version`

View the version of `jvr` itself.

#### 2.1.5`Open`

Open the directory where `jvr` `HOME` is located, if applicable.