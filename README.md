# Yield Lang

YieldLang is a simple work-in-progress functional programming language written in Rust with LLVM as the backend. It aims to provide a concise and expressive syntax for functional programming, while leveraging the performance benefits of LLVM.

## Building and Running Examples

To build and run examples in YieldLang, follow these steps:

1. Make sure you have Rust and LLVM installed on your system.

2. Clone the YieldLang repository:

    ```shell
    git clone https://github.com/luizgabriel/YieldLang.git
    ```

3. Navigate to the project directory:

    ```shell
    cd YieldLang
    ```

4. Run an example file using the following command:

    ```shell
    cargo run -q -- examples/test.y -o test.ll && lli test.ll
    ```

    This command compiles the `test.y` file to LLVM IR (`test.ll`) and then executes it using the LLVM interpreter (`lli`).

    Replace `examples/test.y` with the path to your desired example file.

5. You should see the output of the executed YieldLang program.

Feel free to explore the `examples` directory for more example files.

## Contributing

If you're interested in contributing to YieldLang, please refer to the [CONTRIBUTING.md](CONTRIBUTING.md) file for guidelines.

## License

YieldLang is licensed under the [Apache License 2.0](LICENSE).
