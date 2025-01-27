# Mini Spreadsheet

A lightweight spreadsheet application supporting basic calculations, functions, and cell references.

## Quick Start

### Try it online !!

You can try out Mini Spreadsheet online at: [https://dorukcem.itch.io/mini-spreadsheet](https://dorukcem.itch.io/mini-spreadsheet)

### Build and run it yourself

#### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- [rustup](https://rustup.rs/)

#### Building from Source

1. Clone the repository.
2. Follow either the web or desktop build instructions below.


#### Web Version

```bash
# Install dependencies
cargo install basic-http-server
rustup target add wasm32-unknown-unknown

# Build and run
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/mini_spreadsheet.wasm .
basic-http-server .
```

#### Desktop Version

```bash
cargo run
```

## Usage Guide

### Basic Interface

- **Cell Editing**: Click any cell to edit its contents.
- **Cell References**: Hold Ctrl and click a cell to reference it in expressions (e.g., `A1`).
- **Content Overflow**: Hover over truncated cells to view full contents.
- **Error Handling**: Hover over errors for detailed descriptions.

### Data Types

| Type       | Description                         | Examples                |
| ---------- | ----------------------------------- | ----------------------- |
| Text       | Plain text strings                  | `Hello`, `World`        |
| Number     | Numeric values (integers or floats) | `42`, `3.14`, `1e-6`    |
| Boolean    | True/false values                   | `TRUE`, `FALSE`         |
| Expression | Formula starting with `=`           | `=A1+B1`, `=sum(A1:A4)` |

### Expression Syntax

- **Basic Operations**: Support standard mathematical operators (`+`, `-`, `*`, `/`).
- **Text Literals**: Use double quotes (e.g., `="Hello"+"World"`).
- **Cell References**: Direct (e.g., `A1`) or ranges (e.g., `A1:A4`).
- **Range Limits**: Maximum 100 rows and columns.

### Built-in Functions

#### Mathematical Functions

- `sum(args...)`: Sum of numeric values.
- `product(args...)`: Product of numeric values.
- `average(args...)`: Arithmetic mean.
- `max(args...)`: Maximum value.
- `min(args...)`: Minimum value.
- `pow(base, exponent)`: Power calculation.
- `round(number)`: Round to nearest integer.

#### Utility Functions

- `count(args...)`: Count numeric values.
- `length(text)`: String length.
- `if(condition, true_value, false_value)`: Conditional logic.

#### Function Usage Examples

```xls
= sum(A1:A4)           
= average(1, 2, A1)    
= if(A1>10, "High", "Low")  
```

