// In this file goes all the stuff needed for the schema decoder to work.
extern crate serde_json;

use std::path::PathBuf;
use std::error;
use std::fs::File;
use std::io::{
    Write, Error, ErrorKind
};

use super::schemas_importer;

/// This struct holds the entire schema for the currently selected game (by "game" I mean the PackFile
/// Type).
/// It has:
/// - game: the game for what the loaded definitions are intended.
/// - version: custom variable to keep track of the updates to the schema.
/// - tables_definition: the actual definitions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schema {
    pub game: String,
    pub version: u32,
    pub tables_definitions: Vec<TableDefinitions>,
}

/// This struct holds the definitions for a table. It has:
/// - name: the name of the table.
/// - versions: the different versions this table has.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TableDefinitions {
    pub name: String,
    pub versions: Vec<TableDefinition>,
}

/// This struct holds the definitions for a version of a table. It has:
/// - version: the version of the table these definitions are for.
/// - fields: the different fields this table has.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TableDefinition {
    pub version: u32,
    pub fields: Vec<Field>,
}

/// This struct holds the type of a field of a table. It has:
/// - field_name: the name of the field.
/// - field_is_key: true if the field is a key field and his column needs to be put in the beginning of the TreeView.
/// - field_is_reference: if this field is a reference of another, this has (table name, field name).
/// - field_type: the type of the field.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Field {
    pub field_name: String,
    pub field_type: FieldType,
    pub field_is_key: bool,
    pub field_is_reference: Option<(String, String)>,
    pub field_description: String,
}

/// Enum FieldType: This enum is used to define the possible types of a field in the schema.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FieldType {
    Boolean,
    Float,
    Integer,
    StringU8,
    StringU16,
    OptionalStringU8,
    OptionalStringU16,
}

/// Implementation of "Schema"
impl Schema {

    /// This function creates a new schema. It should only be needed to create the first table definition
    /// of a game, as the rest will continue using the same schema.
    pub fn new(game: String) -> Schema {
        let version = 1u32;
        let tables_definitions = vec![];

        Schema {
            game,
            version,
            tables_definitions,
        }
    }

    /// This function adds a new TableDefinitions to the schema. This checks if that table definitions
    /// already exists, and replace it in that case.
    pub fn add_table_definitions(&mut self, table_definitions: TableDefinitions) {

        let name = table_definitions.name.to_owned();
        let mut index_name = 0;
        let mut index_found = false;
        for (index, definitions) in self.tables_definitions.iter().enumerate() {
            if definitions.name == name {
                index_name = index;
                index_found = true;
                break;
            }
        }
        if index_found {
            self.tables_definitions.remove(index_name);
            self.tables_definitions.insert(index_name, table_definitions);
        }
        else {
            self.tables_definitions.push(table_definitions);
        }
    }

    /// This functions returns the index of the definitions for a table.
    pub fn get_table_definitions(&self, table_name: &str) -> Option<usize> {
        for (index, table_definitions) in self.tables_definitions.iter().enumerate() {
            if table_definitions.name == table_name {
               return Some(index);
            }
        }
        None
    }

    /// This function takes an schema file and reads it into a "Schema" object.
    pub fn load() -> Result<Schema, Error> {
        let schema_file = File::open("schema_wh2.json")?;
        let schema = serde_json::from_reader(schema_file)?;
        Ok(schema)
    }

    /// This function takes an "Schema" object and saves it into a schema file.
    pub fn save(schema: &Schema) -> Result<(), Error> {
        let schema_json = serde_json::to_string_pretty(schema);
        match File::create(PathBuf::from("schema_wh2.json")) {
            Ok(mut file) => {
                match file.write_all(&schema_json.unwrap().as_bytes()) {
                    Ok(_) => Ok(()),
                    Err(error) => Err(Error::new(ErrorKind::Other, error::Error::description(&error).to_string()))
                }
            },
            Err(error) => Err(Error::new(ErrorKind::Other, error::Error::description(&error).to_string()))
        }
    }
}

/// Implementation of "TableDefinitions"
impl TableDefinitions {

    /// This function creates a new table definition. We need to call it when we don't have a definition
    /// of the table we are trying to decode.
    pub fn new(name: &str) -> TableDefinitions {
        let name = name.to_string();
        let versions = vec![];

        TableDefinitions {
            name,
            versions,
        }
    }

    /// This functions returns the index of the definitions for a table.
    pub fn get_table_version(&self, table_version: u32) -> Option<usize> {
        for (index, table) in self.versions.iter().enumerate() {
            if table.version == table_version {
                return Some(index);
            }
        }
        None
    }

    /// This functions adds a new TableDefinition to the list. This checks if that version of the table
    /// already exists, and replace it in that case.
    pub fn add_table_definition(&mut self, table_definition: TableDefinition) {
        let version = table_definition.version;
        let mut index_version = 0;
        let mut index_found = false;
        for (index, definition) in self.versions.iter().enumerate() {
            if definition.version == version {
                index_version = index;
                index_found = true;
                break;
            }
        }
        if index_found {
            self.versions.remove(index_version);
            self.versions.insert( index_version, table_definition);
        }
        else {
            self.versions.push(table_definition);
        }
    }
}

/// Implementation of "TableDefinition"
impl TableDefinition {

    /// This function creates a new table definition. We need to call it when we don't have a definition
    /// of the table we are trying to decode with the version we have.
    pub fn new(version: u32) -> TableDefinition {
        let fields = vec![];

        TableDefinition {
            version,
            fields,
        }
    }

    /// This function creates a new table definition from an imported definition from the assembly kit.
    /// Note that this import the loc fields (they need to be removed manually later) and it doesn't
    /// import the version (this... I think I can do some trick for it).
    pub fn new_from_assembly_kit(imported_table_definition: schemas_importer::root, version: u32, table_name: String) -> TableDefinition {
        let mut fields = vec![];
        for field in imported_table_definition.field.iter() {

            // First, we need to disable a number of known fields that are not in the final tables. We
            // check if the current field is one of them, and ignore it if it's.
            if field.name == "game_expansion_key" ||
                field.name == "localised_text" ||
                field.name == "localised_name" ||
                field.name == "localised_tooltip" ||
                field.name == "description" ||
                field.name == "objectives_team_1" ||
                field.name == "objectives_team_2" ||
                field.name == "short_description_text" ||
                field.name == "historical_description_text" ||
                field.name == "strengths_weaknesses_text" ||
                field.name == "onscreen_name" ||
                field.name == "onscreen_description" ||
                field.name == "on_screen_name" ||
                field.name == "on_screen_description" ||
                field.name == "on_screen_target" {
                continue;
            }


            let field_name = field.name.to_owned();
            let field_is_key = if field.primary_key == "1" {true} else {false};

            let field_is_reference = if field.column_source_table != None {
                Some((field.column_source_table.clone().unwrap().to_owned(), field.column_source_column.clone().unwrap()[0].to_owned()))
            }
            else {None};

            let field_type = match &*field.field_type {
                "yesno" => FieldType::Boolean,
                "single" | "decimal" | "double" => FieldType::Float,
                "integer" | "autonumber" => {

                    // In Warhammer 2 these tables are wrong in the definition schema.
                    if table_name.starts_with("_kv") {
                        FieldType::Float
                    }
                    else {
                        FieldType::Integer
                    }
                },
                "text" => {

                    // Key fields are ALWAYS REQUIRED. This fixes it's detection.
                    if field_name == "key" {
                        FieldType::StringU8
                    }
                    else {
                        match &*field.required {
                            "1" => {
                                // In Warhammer 2 this table has his "value" field broken.
                                if table_name == "_kv_winds_of_magic_params_tables" && field_name == "value" {
                                    FieldType::Float
                                }
                                else {
                                    FieldType::StringU8
                                }
                            },
                            "0" => FieldType::OptionalStringU8,
                            _ => FieldType::Integer,
                        }
                    }
                }
                _ => FieldType::Integer,

            };

            let field_description = match field.field_description {
                Some(ref description) => description.to_owned(),
                None => String::new(),
            };

            let new_field = Field::new(
                field_name,
                field_type,
                field_is_key,
                field_is_reference,
                field_description
            );
            fields.push(new_field);
        }

        TableDefinition {
            version,
            fields,
        }
    }

    /// This function adds a field to a table. It's just to make it easy to interact with, so we don't
    /// need to call the "Field" stuff manually.
    pub fn add_field(&mut self, field_name: String, field_type: FieldType, field_is_key: bool, field_is_reference: Option<(String, String)>) {
        self.fields.push(Field::new(field_name, field_type, field_is_key, field_is_reference, String::new()));
    }
}

/// Implementation of "Field"
impl Field {

    /// This function creates a new table definition. We need to call it when we don't have a definition
    /// of the table we are trying to decode with the version we have.
    pub fn new(field_name: String, field_type: FieldType, field_is_key: bool, field_is_reference: Option<(String, String)>, field_description: String) -> Field {

        Field {
            field_name,
            field_type,
            field_is_key,
            field_is_reference,
            field_description
        }
    }
}