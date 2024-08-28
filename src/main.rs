use clap::{Arg, Command};
use serde::Serialize;
use sqlx::{MssqlPool, Row};
use std::fs::File;
use std::io::Write;

#[derive(Serialize)]
struct Column {
    name: String,
    data_type: String,
}

#[derive(Serialize)]
struct Table {
    name: String,
    columns: Vec<Column>,
}

#[derive(Serialize)]
struct Reference {
    table: String,
    column: String,
    referenced_table: String,
    referenced_column: String,
}

#[derive(Serialize)]
struct DatabaseSchema {
    tables: Vec<Table>,
    references: Vec<Reference>,
}

async fn get_tables(pool: &MssqlPool) -> Result<Vec<Table>, Box<dyn std::error::Error>> {
    let mut tables = Vec::new();
    let rows = sqlx::query("SELECT TABLE_NAME FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_TYPE = 'BASE TABLE'")
        .fetch_all(pool)
        .await?;

    for row in rows {
        let table_name: String = row.try_get("TABLE_NAME")?;
        let columns = get_columns(pool, &table_name).await?;
        tables.push(Table {
            name: table_name,
            columns,
        });
    }
    Ok(tables)
}

async fn get_columns(pool: &MssqlPool, table_name: &str) -> Result<Vec<Column>, Box<dyn std::error::Error>> {
    let mut columns = Vec::new();
    let query = format!("SELECT COLUMN_NAME, DATA_TYPE FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_NAME = '{}'", table_name);
    let rows = sqlx::query(&query)
        .fetch_all(pool)
        .await?;

    for row in rows {
        let column_name: String = row.try_get("COLUMN_NAME")?;
        let data_type: String = row.try_get("DATA_TYPE")?;
        columns.push(Column {
            name: column_name,
            data_type: data_type,
        });
    }
    Ok(columns)
}

async fn get_references(pool: &MssqlPool) -> Result<Vec<Reference>, Box<dyn std::error::Error>> {
    let mut references = Vec::new();
    let query = "
        SELECT 
            tp.name AS TABLE_NAME,
            cp.name AS COLUMN_NAME,
            tr.name AS REFERENCED_TABLE_NAME,
            cr.name AS REFERENCED_COLUMN_NAME
        FROM 
            sys.foreign_keys AS fk
        INNER JOIN 
            sys.foreign_key_columns AS fkc ON fk.object_id = fkc.constraint_object_id
        INNER JOIN 
            sys.tables AS tp ON fkc.parent_object_id = tp.object_id
        INNER JOIN 
            sys.columns AS cp ON fkc.parent_object_id = cp.object_id AND fkc.parent_column_id = cp.column_id
        INNER JOIN 
            sys.tables AS tr ON fkc.referenced_object_id = tr.object_id
        INNER JOIN 
            sys.columns AS cr ON fkc.referenced_object_id = cr.object_id AND fkc.referenced_column_id = cr.column_id";

    let rows = sqlx::query(query)
        .fetch_all(pool)
        .await?;

    for row in rows {
        let table: String = row.try_get("TABLE_NAME")?;
        let column: String = row.try_get("COLUMN_NAME")?;
        let referenced_table: String = row.try_get("REFERENCED_TABLE_NAME")?;
        let referenced_column: String = row.try_get("REFERENCED_COLUMN_NAME")?;
        references.push(Reference {
            table,
            column,
            referenced_table,
            referenced_column,
        });
    }
    Ok(references)
}

fn generate_plantuml(schema: &DatabaseSchema) -> String {
    let mut plantuml = String::new();
    plantuml.push_str("@startuml\n");
    for table in &schema.tables {
        plantuml.push_str(&format!("class {} {{\n", table.name));
        for column in &table.columns {
            plantuml.push_str(&format!("  {} : {}\n", column.name, column.data_type));
        }
        plantuml.push_str("}\n");
    }
    for reference in &schema.references {
        plantuml.push_str(&format!(
            "{}::{} --> {}::{} : {}\n",
            reference.table, reference.column, reference.referenced_table, reference.referenced_column, reference.column
        ));
    }
    plantuml.push_str("@enduml\n");
    plantuml
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("TSQLDiagramGenerator")
        .version("1.0")
        .author("Tyler Maginnis <maginnist@gmail.com>")
        .about("Generates a TSQL database diagram")
        .arg(
            Arg::new("ip_address")
                .short('i')
                .long("ip_address")
                .help("Sets the IP address of the SQL server")
                .required(true)
        )
        .arg(
            Arg::new("username")
                .short('u')
                .long("username")
                .help("Sets the username for the SQL server")
                .required(true)
        )
        .arg(
            Arg::new("password")
                .short('p')
                .long("password")
                .help("Sets the password for the SQL server")
                .required(true)
        )
        .arg(
            Arg::new("initial_catalog")
                .short('c')
                .long("initial_catalog")
                .help("Sets the initial catalog for the SQL server")
                .required(true)
        )
        .get_matches();

    let ip_address = matches.get_one::<String>("ip_address").unwrap();
    let username = matches.get_one::<String>("username").unwrap();
    let password = matches.get_one::<String>("password").unwrap();
    let initial_catalog = matches.get_one::<String>("initial_catalog").unwrap();

    // Configure the connection with a timeout
    let connection_string = format!(
        "mssql://{}:{}@{}:1433/{}?trustservercertificate=true&connect_timeout=30",
        username, password, ip_address, initial_catalog
    );
    let pool = MssqlPool::connect(&connection_string).await?;

    // Get the database schema
    let tables = get_tables(&pool).await?;
    let references = get_references(&pool).await?;
    let schema = DatabaseSchema { tables, references };

    // Generate PlantUML script
    let plantuml_script = generate_plantuml(&schema);

    // Save PlantUML script to a file
    let mut file = File::create("schema.puml")?;
    file.write_all(plantuml_script.as_bytes())?;

    println!("PlantUML script generated and saved to schema.puml");

    Ok(())
}