# How to use the Fuzzer

Using the fuzzer is quite simple. You just need Rust, a SuiMove package and to configure the fuzzer.

## Requirements

You'll need the latest version of **Rust** to run the fuzzer. We recommend using [Rustup](https://rustup.rs/) to install it.

## Steps

Before you can start the fuzzer, you need to prepare a few things, the **fuzzinglabs** package will be used as an example in this documentation.

### 1. Compile the SuiMove package you want to fuzz

Go in the root (*./examples/fuzzinglabs_package* in our case) directory of the package and run the following command:

```bash
$ sui move build
```
If everything works properly, you should have the following result:

```
UPDATING GIT DEPENDENCY https://github.com/MystenLabs/sui.git
INCLUDING DEPENDENCY Sui
INCLUDING DEPENDENCY MoveStdlib
BUILDING fuzzinglabs_package
```

### 2. Configuring the fuzzer

Now, you'll need to tell the fuzzer where to find the compiled module that you want to fuzz and configure a few parameters.

To do so you need to edit the *config.json* file (or create a new file) located at the root of the repository.

```json
{
  "use_ui": true, // Do you want the nice UI or not ?
  "nb_threads": 8, // The number of threads used by the fuzzer
  "seed": 4242, // The inital seed
  "contract_file": "./examples/fuzzinglabs_package/build/fuzzinglabs_package/bytecode_modules/fuzzinglabs_module.mv", // The path to the compiled module
  "execs_before_cov_update": 10000, // When the coverage is shared between the threads (don't modify if you don't know why)
  "corpus_dir": "./corpus", // Path to where the corpus will be written (milestone 3)
  "crashes_dir": "./crashes", // Path to where the crashfiles will be written
  "fuzz_functions_prefix": "fuzz_" // Fuzzing functions prefix (can be listed by the fuzzer)
}
```
Edit the file to meet your needs.

### 3. Start the fuzzer

You can now start the fuzzer ! It can be done using this command:

```bash
$ make CONFIG_PATH="./config.json" TARGET_MODULE="fuzzinglabs_module" TARGET_FUNCTION="fuzzinglabs"
```

The fuzzer will automatically detect the parameters of the fuzzed function.

You should get something like this (if **use_ui** is set to **true**):

![screenshot](./imgs/screenshot1.png)

## More options

The fuzzer has a few more options.

### Listing fuzzing functions

You can list all the available fuzzing functions in the given module using the following command:

```bash
$ make list_functions CONFIG_PATH="./config.json"
```

### Detectors

The detectors are part of the fuzzer that can be enabled or not. They are used to check if the execution of the fuzzed function matches a knowned vulnerability.

In order to tell the fuzzer to activated them you have to use the following command:

```bash
$ make CONFIG_PATH="./config.json" TARGET_MODULE="fuzzinglabs_module" TARGET_FUNCTION="fuzzinglabs" DETECTORS="basic-op-code-detector"
```

You can enable multiple detectors by seprating them using a comma. Like so:

```bash
DETECTORS="basic-op-code-detector,another-one,and-another"
```
