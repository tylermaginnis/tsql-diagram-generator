# TSQL Diagram Generator

## Description

TSQL Diagram Generator is a Rust-based application designed to generate a TSQL database diagram from a Microsoft SQL Server database. It connects to the database, retrieves the schema information, and generates a PlantUML script that represents the database schema, including tables, columns, and references.

## Features

- Connects to a Microsoft SQL Server database
- Retrieves table and column information
- Retrieves foreign key references
- Generates a PlantUML script representing the database schema
- Saves the PlantUML script to a file

## Installation

To install this application, you need to have Rust and Cargo installed on your system. Follow the steps below to get started:

1. Clone the repository:
    ```sh
    git clone https://github.com/tylermaginnis/tsql_diagram_generator.git
    ```
2. Navigate to the project directory:
    ```sh
    cd tsql_diagram_generator
    ```
3. Build the project:
    ```sh
    cargo build --release
    ```
4. Run the application:
    ```sh
    cargo run -- --ip_address <IP_ADDRESS> --username <USERNAME> --password <PASSWORD> --initial_catalog <INITIAL_CATALOG>
    ```

## Usage

To use the TSQL Diagram Generator, you need to provide the IP address, username, password, and initial catalog of the SQL server. The application will connect to the database, retrieve the schema information, and generate a PlantUML script that represents the database schema.

Example usage:

```sh
cargo run -- --ip_address 192.168.1.1 --username admin --password secret --initial_catalog my_database
```

This will connect to the SQL server at 192.168.1.1, with the username "admin", password "secret", and initial catalog "my_database". The application will retrieve the schema information, and generate a PlantUML script that represents the database schema.

## License

This project is licensed under the MIT License. See the LICENSE file for more details.