# How to Use
## UI
You can edit cells by clicking on them. To add a reference to another cell (only when writing expressions), such as 'A1', you can click on the cell you want to reference while holding down the Ctrl key. You can hover over any error to see a detailed description. When the contents of the cell does not fit the cell, the content will be truncated, you can see the full contents of the cell by hovering over it.

## Syntax

| Type       | Description                                                                                             | Examples                             |
| ---------- | ------------------------------------------------------------------------------------------------------- | ------------------------------------ |
| Text       | Just a human readiable text.                                                                            | `A`, `B`, `C`, etc                   |
| Number     | Anything that can be parsed as a float                                                                  | `1`, `2.0`, `1e-6`, etc              |
| Boolean    | TRUE or FALSE                                                                                           | `TRUE`, `FALSE`                      |
| Expression | Always starts with `=`. Excel style math expression that involves other cells, functions and operations | `=A1+B1`, `=42*2`, `=pow(A1, 2)` etc |

### Value Types
Boolean, number, and text are value types which can be the final value of a cell.

### Expressions
Expressions evaluate to a value type or an error. Errors may arise from unparsable expressions (such as = 1  2), invalid operations (such as trying to add text to a number), etc. Text literals may be used in expressions using double quotes, such as = "hello" + "world". 

### Ranges
Range expressions may appear inside of function calls in this `FROM_CELL : TO_CELL` form. They can be used to operate on a range of cells. For example: `=sum(A1:A4)` would sum the first 4 elements of the first column.

### Functions
Functions perform a single action and evaluate to a result. They may take arguments separated by commas. Some functions may require a specific number of arguments, while others can take any number of arguments (for example, we can say sum(1, 2, A1:C5)). Functions always start with lowercase letters. 

### Built-in Functions

1. **sum**  
   Returns the sum of all numeric arguments.
2. **product**  
   Returns the product of all numeric arguments.
3. **max**  
   Returns the maximum value among the numeric arguments.
4. **min**  
   Returns the minimum value among the numeric arguments.
5. **average**  
   Returns the average (arithmetic mean) of numeric arguments.
6. **count**  
   Returns the count of numeric arguments.
7. **length**  
   Returns the length of a text argument.
8. **if**  
   Returns the second argument if the first argument evaluates to `true`, otherwise returns the third argument.
9. **round**  
   Rounds a single numeric argument to the nearest whole number.
10. **pow**  
    Returns the first numeric argument raised to the power of the second numeric argument.

# Build and Run
You will need to have installed rustup and cargo for to build this project: [Install Rustup and Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

## How to build an run for the web
In the root folder of this project

`cargo install basic-http-server`

`rustup target add wasm32-unknown-unknown`

`cargo build --release --target wasm32-unknown-unknown && cp target/wasm32-unknown-unknown/release/mini_spreadsheet.wasm . && basic-http-server .`

## How to Build and Run Locally (as a Desktop App)
In the root folder of this project

`cargo run`