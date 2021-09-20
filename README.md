# **RofiTodo**

![Build](https://github.com/Any0ne22/RofiTodo/actions/workflows/rust.yml/badge.svg)
![LastCommit](https://img.shields.io/github/last-commit/Any0ne22/RofiTodo)
![LastRelease](https://img.shields.io/github/v/release/Any0ne22/RofiTodo)

A to-do list using Rofi compatible with [todo.txt specification](https://github.com/todotxt/todo.txt)

## **Installation**

1) Clone this repository
2) Go to `Rofitodo`
3) Run the following commands

    ```bash
    autoreconf -si
    ./configure
    make
    make install
    ```

    You can also build RofiTodo using cargo with

    ```bash
    cargo build --release
    ```

    and put the `rofitodo` executable (located in `target/release`) where you want.

## **Usage**

- Print help :

    ```bash
    rofitodo -h
    ```

- Specify a tasklist-file:

    ```bash
    rofitodo -c path/to/your/todolist
    ```

    or

    ```bash
    rofitodo --config path/to/your/todolist
    ```

- Do not load Rofi configuration, use default values :

    ```bash
    rofitodo --no-config
    ```

- Set filter to be case insensitive :

    ```bash
    rofitodo -i
    ```

    or

    ```bash
    rofitodo --case-insensitive
    ```

- Set the default sorting order between creation date (`creation`), lexicographic (`content`), due date (`due`) and priority (`priority`):

    ```bash
    rofitodo -s creation
    ```

    or

    ```bash
    rofitodo --sort creation
    ```

- Print version :

    ```bash
    rofitodo -V
    ```
