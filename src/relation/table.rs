use std::{fmt::{self, Display}, collections::{HashSet, HashMap}};
use core::hash::Hash;

use log::info;
use mysql::{Row};
use regex::Regex;

use crate::{data_base::DataBase, sql::{SQL, QDL, DDL, QML}};

use super::RelationMethods;

/// A struct representing a table in a relational database
#[derive(Clone, Debug)]
pub struct Table{
    /// The name of the table.
    pub name: String,
    /// A vector of `Attribute`s representing the columns of the table.
    pub attributes: Vec<Attribute>,
    /// The index of the primary key attribute in the `attributes` vector, if one exists.
    pub primary_key: Option<usize>,
}

impl Table {
    /// Returns a `Table` with the given name, created from the database.
    /// Return None if the table does not exist.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of an already existing table.
    ///
    /// # Examples
    ///
    /// ```
    /// let table = Table::from_db("employees").unwrap();
    /// ```
    pub fn from_db(table_name: &str) -> Option<Table> {
        //println!("\n\n");
        match DataBase::from_env() {
            Ok(db) => {
                        
                let mut primary_key : Option<usize> = None;
                let mut col_num: usize = 0;
                
                //println!("{}", table_name);
                let attr : Vec<(Option<Attribute>, bool)> = db.execute(
                    &SQL::new(&format!("SHOW FULL COLUMNS FROM {}", table_name)).unwrap(),
                    |row| {
                        match row {
                            Ok(column) => {
                                info!("load row:{:?}", column);
                                let primary_key_str: String = column.get(4).unwrap();

                                (Attribute::from_row(column, table_name), &primary_key_str == "PRI")
                            },
                            Err(_err) => {
                                todo!()
                            }
                        }
                    }
                ).unwrap();

                let attr: Vec<Attribute> = attr.iter()
                .filter_map(
                    |val| {
                        match val {
                            (None, _val) => None,
                            (Some(val), true) => {
                                primary_key = Some(col_num.clone());
                                Some(val.clone())
                            },
                            (Some(val), false) => {
                                col_num+=1;
                                Some(val.clone())
                            },
                        }
                    }
                ).collect();

                Some(
                    Table{
                        name: table_name.to_string(),
                        attributes: attr,
                        primary_key: primary_key
                    }
                )
            },
            Err(_err) => {
                None
            }
        }
    }

    /// Returns a vector of foreign key tuples for the table.
    ///
    /// The tuples contain the name of the table and the name of the attribute that the foreign key references.
    ///
    /// # Examples
    ///
    /// ```
    /// let foreign_keys = table.get_foreign_keys().unwrap();
    /// ```
    pub fn get_foreign_keys(&self) -> Option<Vec<(String, String)>> {

        let foreign_key: Vec<(String, String)> = self.attributes
            .iter()
            .filter(|a| {//filter out all attributes without foreign key constraint
                for constraint in &a.constraint{
                    if let Constraint::ForeignKey { .. } = constraint {
                        return true
                    }
                }
                return false
            })
            .map(|a| {//turn a into foreign key
                for constraint in &a.constraint{
                    if let Constraint::ForeignKey { table_name, attribute_name  } = constraint {
                        return (table_name.clone(), attribute_name.clone())
                    }
                }
                panic!()
            })
            .collect();

        if foreign_key.len() == 0 {
            return None
        }

        Some(foreign_key)
    }

    /// Returns a `QML` representing an `INSERT` statement for the table with the given values.
    ///
    /// # Arguments
    ///
    /// * `values` - A `HashMap` of column names and values to insert into the table.
    ///
    /// # Examples
    ///
    /// Creating an insertion statement where all the columns have an inserted value
    /// ```rust
    /// let table = Table{
    ///     name: String::from("table_1"),
    ///     attributes: vec![
    ///         Attribute{
    ///             name: String::from("PersonID"),
    ///             data_type: AttributeType::Int(16),
    ///             constraint: HashSet::new()
    ///         },
    ///         Attribute{
    ///             name: String::from("LastName"),
    ///             data_type: AttributeType::VarChar(255),
    ///             constraint: HashSet::new()
    ///         },
    ///         Attribute{
    ///             name: String::from("FirstName"),
    ///             data_type: AttributeType::VarChar(255),
    ///             constraint: HashSet::new()
    ///         },
    ///         Attribute{
    ///             name: String::from("Address"),
    ///             data_type: AttributeType::VarChar(255),
    ///             constraint: HashSet::new()
    ///         },
    ///         Attribute{
    ///             name: String::from("City"),
    ///             data_type: AttributeType::VarChar(255),
    ///             constraint: HashSet::new()
    ///         },
    ///     ],
    ///     primary_key: None,
    /// };
    /// 
    /// let mut values = HashMap::new();
    /// 
    /// values.insert(String::from("PersonID"), String::from("23"));
    /// values.insert(String::from("LastName"), String::from("'Doe'"));
    /// values.insert(String::from("FirstName"), String::from("'John'"));
    /// values.insert(String::from("Address"), String::from("'1st Street'"));
    /// values.insert(String::from("City"), String::from("'Night City'"));
    /// 
    /// let actual = table.insert(&values);
    /// assert_eq!(actual, Some(QML(String::from("INSERT INTO table_1(PersonID,LastName,FirstName,Address,City) VALUES (23,'Doe','John','1st Street','Night City')"))));
    /// ```
    ///
    /// Creating an insertion statement where some the columns have an inserted value
    /// ```rust
    /// let table = Table{
    ///     name: String::from("table_1"),
    ///     attributes: vec![
    ///         Attribute{
    ///             name: String::from("PersonID"),
    ///             data_type: AttributeType::Int(16),
    ///             constraint: HashSet::new()
    ///         },
    ///         Attribute{
    ///             name: String::from("LastName"),
    ///             data_type: AttributeType::VarChar(255),
    ///             constraint: HashSet::new()
    ///         },
    ///         Attribute{
    ///             name: String::from("FirstName"),
    ///             data_type: AttributeType::VarChar(255),
    ///             constraint: HashSet::new()
    ///         },
    ///         Attribute{
    ///             name: String::from("Address"),
    ///             data_type: AttributeType::VarChar(255),
    ///             constraint: HashSet::new()
    ///         },
    ///         Attribute{
    ///             name: String::from("City"),
    ///             data_type: AttributeType::VarChar(255),
    ///             constraint: HashSet::new()
    ///         },
    ///     ],
    ///     primary_key: None,
    /// };
    /// 
    /// let mut values = HashMap::new();
    /// 
    /// values.insert(String::from("PersonID"), String::from("23"));
    /// values.insert(String::from("LastName"), String::from("'Doe'"));
    /// values.insert(String::from("FirstName"), String::from("'John'"));
    /// 
    /// let actual = table.insert(&values);
    /// assert_eq!(actual, Some(QML(String::from("INSERT INTO table_1(PersonID,LastName,FirstName) VALUES (23,'Doe','John')"))));
    /// ```
    ///
    /// Failed insertion creation results in `Option::None` being returned
    /// ```rust
    /// let table = Table{
    ///     name: String::from("table_1"),
    ///     attributes: vec![
    ///         Attribute{
    ///             name: String::from("PersonID"),
    ///             data_type: AttributeType::Int(16),
    ///             constraint: HashSet::new()
    ///         },
    ///         Attribute{
    ///             name: String::from("LastName"),
    ///             data_type: AttributeType::VarChar(255),
    ///             constraint: HashSet::new()
    ///         },
    ///         Attribute{
    ///             name: String::from("FirstName"),
    ///             data_type: AttributeType::VarChar(255),
    ///             constraint: HashSet::new()
    ///         },
    ///         Attribute{
    ///             name: String::from("Address"),
    ///             data_type: AttributeType::VarChar(255),
    ///             constraint: HashSet::new()
    ///         },
    ///         Attribute{
    ///             name: String::from("City"),
    ///             data_type: AttributeType::VarChar(255),
    ///             constraint: HashSet::new()
    ///         },
    ///     ],
    ///     primary_key: None,
    /// };
    /// 
    /// let mut values = HashMap::new();
    ///
    /// let actual = table.insert(&values);
    /// 
    /// assert_eq!(actual, None);
    /// ```
    pub fn insert(&self, values: &HashMap<String, String>) -> Option<QML>{//should be turned into Result<SQL, ERROR why couldn't be parsed>
        let (columns, values) = self.attributes
            .iter()
            .filter(|attr| {
                if let Some(_) = values.get(&attr.name) {
                    return true;
                }
                return false;
            })
            .map(|attr| {
                (
                    attr.name.clone(),
                    {
                        match &attr.data_type{
                            // AttributeType::Char(_) |
                            // AttributeType::VarChar(_) |
                            // AttributeType::Binary(_) |
                            // AttributeType::VarBinary(_) |
                            // AttributeType::TinyBlob |
                            // AttributeType::TinyText |
                            // AttributeType::Text(_) |
                            // AttributeType::Blob(_) |
                            // AttributeType::MediumText |
                            // AttributeType::MediumBlob |
                            // AttributeType::LongText |
                            // AttributeType::LongBlob |
                            
                            // AttributeType::Date |
                            // AttributeType::DateTime |
                            // AttributeType::Time => format!("\'{}\'", values.get(&attr.name).unwrap()),

                            _ => values.get(&attr.name).unwrap().to_string()
                        }
                    }
                )
            })
            .fold(
                (String::new(), String::new()),
                |(columns, values), (column, value)| (format!("{},{}", columns, column), format!("{},{}", values, value))
            );

        if ("", "") == (&columns, &values) {
            return None;
        }

        let (m1, m2) = (columns.len(), values.len());

        Some(QML(format!("INSERT INTO {}({}) VALUES ({})", &self.name, &columns[1..m1], &values[1..m2])))
    }

}

impl Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let attr : Vec<String> = self.attributes
            .iter()
            .map(|attr| {
                attr.to_string() 
            })
            .collect();
        
        let attr = attr.join(",");

        match self.primary_key {
            Some(index) => {
                let primary_key = &self.attributes[index].name;

                write!(f, "CREATE TABLE {} ({}, PRIMARY KEY({}))", self.name, attr, primary_key)
            },
            None => write!(f, "CREATE TABLE {} ({})", self.name, attr),
        }
    }
}

impl RelationMethods for Table {
    fn select(&self) -> QDL {
        QDL(format!("SELECT * FROM {}", self.name))
    }
    fn drop(&self) -> DDL{
        DDL(format!("DROP TABLE {}", self.name))
    }
    fn create(&self) -> DDL{
        DDL(self.to_string())
    }
}

/// An attribute in a table of a relational database
#[derive(Clone, Debug)]
pub struct Attribute{
    /// The name of the attribute.
    pub name: String,
    /// The data type of the attribute.
    pub data_type: AttributeType,
    /// A vector of Constraints on the attribute.
    pub constraint: HashSet<Constraint>
}

impl Attribute {
    fn from_row(row: Row, table_name: &str) -> Option<Attribute> {
        let name: String = row.get(0).unwrap();
        let data_type: String = row.get(1).unwrap();

        info!("name:{}\tdata_type:{}", name, data_type);

        let data_type = match AttributeType::from(&data_type.to_ascii_uppercase()) {
            Some(val) => val,
            None => return None,
        };

        Some(
            Attribute {
                name: name.clone(),
                data_type: data_type,
                constraint: {
                    let mut tmp : HashSet<Constraint> = HashSet::new();

                    {
                        let nullable : String = row.get(3).unwrap();
                        
                        if nullable == "NO" {
                            let _result = &tmp.insert(Constraint::NotNull);
                        }
                    }

                    {
                        let auto_inc : String = row.get(6).unwrap();
                        
                        if auto_inc == "auto_increment" {
                            let _result = &tmp.insert(Constraint::AutoIncrement);
                        }
                    }

                    {
                        let key : String = row.get(4).unwrap();
                        
                        if key == "UNI" {
                            let _result = &tmp.insert(Constraint::Unique);
                        }
                        else if key == "MUL" {
                            let db = DataBase::from_env().unwrap();

                            let _tmp: Vec<Constraint> = db.execute(&SQL::new(&format!(r"SHOW CREATE TABLE `{}`;", table_name)).unwrap(), |row| {
                                let command : String = row.unwrap().get(1).unwrap();
                                
                                let tag_check: Regex = Regex::new(&format!("FOREIGN KEY \\(`{}`\\) REFERENCES `([a-zA-Z0-9]+)` \\(`([a-zA-Z0-9]+)`\\)", name)).unwrap();

                                let captures = tag_check.captures(&command).unwrap();

                                Constraint::ForeignKey{
                                    table_name: captures.get(1).unwrap().as_str().to_string(),
                                    attribute_name: captures.get(2).unwrap().as_str().to_string()
                                }
                            }).unwrap();

                            _tmp.iter()
                            .for_each(|constraint| {
                                let _result = &tmp.insert(constraint.clone());
                            });
                        }
                    }

                    tmp
                }
            }
        )
    }

    /// Returns a string representation of the attribute's schema.
    pub fn schema_fmt(&self) -> String {
        format!("{} {}", self.name, self.data_type)
    }
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut foreign_key = None;
        let constraint_str: String = {
            let mut constraints_vec: Vec<String> = Vec::new();
            for c in &self.constraint {
                if let Constraint::ForeignKey { .. } = c {
                    foreign_key = Some(format!("FOREIGN KEY({}) REFERENCES {}", self.name, c.to_string()));
                    continue;
                }

                constraints_vec.push(c.to_string());
            }

            constraints_vec.join(" ")
        };
        let tmp = match constraint_str.len() {
            0 => format!("{} {}", self.name, self.data_type),
            _ => format!("{} {} {}", self.name, self.data_type, constraint_str),
        };

        match foreign_key {
            Some(foreign_key) => write!(f, "{}, {}", tmp, foreign_key),
            None => write!(f, "{}", tmp),
        }
    }
}

/// Constraint defines the restrictions of an attribute
#[derive(Clone, Hash, Eq, Debug)]
pub enum Constraint{
    /// The attribute must not contain a null value.
    NotNull,
    /// The attribute must contain a unique value.
    Unique,
    /// The attribute is a foreign key that references another attribute in a different table.
    ForeignKey{
        /// The name of the table that the foreign key attribute references.
        table_name: String,
        /// The name of the attribute that the foreign key references.
        attribute_name: String
    },
    /// The attribute is an auto-incrementing integer.
    AutoIncrement,
}
impl PartialEq for Constraint {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // (
            //     Self::ForeignKey { .. },
            //     Self::ForeignKey { .. }
            // ) => true,
            
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl fmt::Display for Constraint{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Constraint::NotNull => write!(f, "Not Null"),
            Constraint::Unique => write!(f, "Unique"),
            Constraint::ForeignKey{table_name: table,attribute_name: attr} => write!(f, "{}({})", table, attr),
            Constraint::AutoIncrement => write!(f, "Auto_increment"),
        }
    }
}

/// AttributeType defines every type of MySQL datatype
#[derive(Clone, Debug)]
pub enum AttributeType{
    //string data types
    Char(u8),
    VarChar(u16),
    Binary(u8),
    VarBinary(u16),
    TinyBlob,
    TinyText,
    Text,
    Blob(u16),
    MediumText,
    MediumBlob,
    LongText,
    LongBlob,
    Enum{val: Vec<String>},
    Set{val: Vec<AttributeType>},

    //numeric data types
    Bit(u8),
    TinyInt(u8),
    Bool,
    Boolean,
    SmallInt(u8),
    MediumInt(u8),
    Int(u8),
    BigInt(u8),
    Float(u8),
    Decimal(u8, u8),

    //Date time
    Date,
    DateTime,
    TimeStamp,
    Time,
    Year
}

macro_rules! regex_check {
    ($regex_expr : literal, $raw_str: expr, $output_variant : ident, $parse_type_1: ty, $parse_type_2: ty) => {
        {
            let check = Regex::new($regex_expr).unwrap();

            if let Some(tmp) = check.find($raw_str) {
                if tmp.start() == 0 {
                    let size = check.captures($raw_str).unwrap();

                    let tmp_1 = size.get(1).unwrap().as_str();
                    let tmp_2 = size.get(2).unwrap().as_str();

                    return Some(AttributeType::$output_variant(tmp_1.parse::<$parse_type_1>().unwrap(), tmp_2.parse::<$parse_type_2>().unwrap()))
                }
            }
        }
    };
    ($regex_expr : literal, $raw_str: expr, $output_variant : ident, $parse_type: ty) => {
        {
            let check = Regex::new($regex_expr).unwrap();

            if let Some(tmp) = check.find($raw_str) {
                if tmp.start() == 0 {
                    let size = check.captures($raw_str).unwrap();

                    let tmp = size.get(1).unwrap().as_str();

                    return Some(AttributeType::$output_variant(tmp.parse::<$parse_type>().unwrap()))
                }
            }
        }
    };
    ($regex_expr : literal, $raw_str: expr, $output_variant : ident) => {
        {
            let check = Regex::new($regex_expr).unwrap();

            if let Some(tmp) = check.find($raw_str) {
                if tmp.start() == 0 {
                    return Some(AttributeType::$output_variant)
                }
            }
        }
    };
}

impl AttributeType {
    /// Returns the `AttributeType` variant corresponding to the given string.
    fn from(raw_str: &str) -> Option<AttributeType> {
        regex_check!(r"CHAR\((\d+)\)", raw_str, Char, u8);
        regex_check!(r"VARCHAR\((\d+)\)", raw_str, VarChar, u16);
        regex_check!(r"BINARY\((\d+)\)", raw_str, Binary, u8);
        regex_check!(r"VARBINARY\((\d+)\)", raw_str, VarBinary, u16);
        regex_check!(r"TINYBLOB", raw_str, TinyBlob);
        regex_check!(r"TINYTEXT", raw_str, TinyText);
        regex_check!(r"TEXT", raw_str, Text);
        regex_check!(r"BLOB\((\d+)\)", raw_str, Blob, u16);
        regex_check!(r"MEDIUMTEXT", raw_str, MediumText);
        regex_check!(r"LONGTEXT", raw_str, LongText);
        regex_check!(r"LONGBLOB", raw_str, LongBlob);
        //Enum{val: Vec<String>},
        //Set{val: Vec<AttributeType>},

        regex_check!(r"BIT\((\d+)\)", raw_str, Bit, u8);
        regex_check!(r"TINYINT\((\d+)\)", raw_str, TinyInt, u8);
        regex_check!(r"BOOL", raw_str, Bool);
        regex_check!(r"BOOLEAN", raw_str, Boolean);
        regex_check!(r"SMALLINT\((\d+)\)", raw_str, SmallInt, u8);
        regex_check!(r"MEDIUMINT\((\d+)\)", raw_str, MediumInt, u8);
        regex_check!(r"INT\((\d+)\)", raw_str, Int, u8);
        regex_check!(r"INTEGER\((\d+)\)", raw_str, Int, u8);
        regex_check!(r"BigInt\((\d+)\)", raw_str, BigInt, u8);
        regex_check!(r"FLOAT\((\d+)\)", raw_str, Float, u8);
        regex_check!(r"DECIMAL\((\d+),(\d+)\)", raw_str, Decimal, u8, u8);

        regex_check!(r"DATE", raw_str, Date);
        regex_check!(r"DATETIME", raw_str, DateTime);
        regex_check!(r"TIMESTAMP", raw_str, TimeStamp);
        regex_check!(r"TIME", raw_str, Time);
        regex_check!(r"YEAR", raw_str, Year);

        return None
    }
}

impl fmt::Display for AttributeType{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            //string data types
            AttributeType::Char(val) => write!(f, "char({})", val),
            AttributeType::VarChar(val) => write!(f, "varchar({})", val),
            AttributeType::Binary(val) => write!(f, "binary({})", val),
            AttributeType::VarBinary(val) => write!(f, "varbinary({})", val),
            AttributeType::TinyBlob => write!(f, "tinyblob"),
            AttributeType::TinyText => write!(f, "tinytext"),
            AttributeType::Text => write!(f, "text"),
            AttributeType::Blob(val) => write!(f, "blob({})", val),
            AttributeType::MediumText => write!(f, "mediumtext"),
            AttributeType::MediumBlob => write!(f, "mediumblob"),
            AttributeType::LongText => write!(f, "longtext"),
            AttributeType::LongBlob => write!(f, "longblob"),
            AttributeType::Enum{..} => todo!(),
            AttributeType::Set{..} => todo!(),

            //numeric data types
            AttributeType::Bit(val) => write!(f, "bit({})", val),
            AttributeType::TinyInt(val) => write!(f, "tinyint({})", val),
            AttributeType::Bool => write!(f, "bool"),
            AttributeType::Boolean => write!(f, "boolean"),
            AttributeType::SmallInt(val) => write!(f, "smallint({})", val),
            AttributeType::MediumInt(val) => write!(f, "mediumint({})", val),
            AttributeType::Int(val) => write!(f, "int({})", val),
            AttributeType::BigInt(val) => write!(f, "bigint({})", val),
            AttributeType::Float(val) => write!(f, "float({})", val),
            AttributeType::Decimal(val_1 , val_2) => write!(f, "decmimal({},{})", val_1, val_2),

            //Date time
            AttributeType::Date => write!(f, "date"),
            AttributeType::DateTime => write!(f, "datetime"),
            AttributeType::TimeStamp => write!(f, "timeStamp"),
            AttributeType::Time => write!(f, "time"),
            AttributeType::Year => write!(f, "year"),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use std::collections::{HashSet, HashMap};

    use crate::{backend::{sql::QML, relation::RelationMethods}, ui::menu::Tab};

    use super::{Table, Attribute, AttributeType, Constraint};

    //table Create statement
    #[test]
    fn create_test_1() {
        let table = Table{
            name: String::from("table_1"),
            attributes: vec![
                Attribute{
                    name: String::from("attr_1"),
                    data_type: AttributeType::Text,
                    constraint: HashSet::from(
                        [
                            Constraint::NotNull,
                            Constraint::Unique
                        ]
                    )
                }
            ],
            primary_key: Some(0),
        };

        assert_eq!(*table.create(), "CREATE TABLE table_1 (attr_1 text Unique Not Null, PRIMARY KEY(attr_1))")
    }

    #[test]
    fn create_test_2() {
        let table = Table{
            name: String::from("table_1"),
            attributes: vec![
                Attribute{
                    name: String::from("attr_1"),
                    data_type: AttributeType::Text,
                    constraint: HashSet::new()
                }
            ],
            primary_key: Some(0),
        };

        assert_eq!(*table.create(), "CREATE TABLE table_1 (attr_1 text, PRIMARY KEY(attr_1))")
    }

    //do more tests

    //table insert statement
    #[test]
    fn insert_test_1() {
        let table = Table{
            name: String::from("table_1"),
            attributes: vec![
                Attribute{
                    name: String::from("PersonID"),
                    data_type: AttributeType::Int(16),
                    constraint: HashSet::new()
                },
                Attribute{
                    name: String::from("LastName"),
                    data_type: AttributeType::VarChar(255),
                    constraint: HashSet::new()
                },
                Attribute{
                    name: String::from("FirstName"),
                    data_type: AttributeType::VarChar(255),
                    constraint: HashSet::new()
                },
                Attribute{
                    name: String::from("Address"),
                    data_type: AttributeType::VarChar(255),
                    constraint: HashSet::new()
                },
                Attribute{
                    name: String::from("City"),
                    data_type: AttributeType::VarChar(255),
                    constraint: HashSet::new()
                },
            ],
            primary_key: None,
        };

        let mut values = HashMap::new();

        values.insert(String::from("PersonID"), String::from("23"));
        values.insert(String::from("LastName"), String::from("'Doe'"));
        values.insert(String::from("FirstName"), String::from("'John'"));
        values.insert(String::from("Address"), String::from("'1st Street'"));
        values.insert(String::from("City"), String::from("'Night City'"));

        let actual = table.insert(&values);

        assert_eq!(actual, Some(QML(String::from("INSERT INTO table_1(PersonID,LastName,FirstName,Address,City) VALUES (23,'Doe','John','1st Street','Night City')"))));
    }

    #[test]
    fn insert_test_2(){
        let table = Table{
            name: String::from("table_1"),
            attributes: vec![
                Attribute{
                    name: String::from("PersonID"),
                    data_type: AttributeType::Int(16),
                    constraint: HashSet::new()
                },
                Attribute{
                    name: String::from("LastName"),
                    data_type: AttributeType::VarChar(255),
                    constraint: HashSet::new()
                },
                Attribute{
                    name: String::from("FirstName"),
                    data_type: AttributeType::VarChar(255),
                    constraint: HashSet::new()
                },
                Attribute{
                    name: String::from("Address"),
                    data_type: AttributeType::VarChar(255),
                    constraint: HashSet::new()
                },
                Attribute{
                    name: String::from("City"),
                    data_type: AttributeType::VarChar(255),
                    constraint: HashSet::new()
                },
            ],
            primary_key: None,
        };

        let mut values = HashMap::new();

        values.insert(String::from("PersonID"), String::from("23"));
        values.insert(String::from("LastName"), String::from("'Doe'"));
        values.insert(String::from("FirstName"), String::from("'John'"));

        let actual = table.insert(&values);

        assert_eq!(actual, Some(QML(String::from("INSERT INTO table_1(PersonID,LastName,FirstName) VALUES (23,'Doe','John')"))));
    }

    #[test]
    fn insert_test_3(){
        let table = Table{
            name: String::from("table_1"),
            attributes: vec![
                Attribute{
                    name: String::from("PersonID"),
                    data_type: AttributeType::Int(16),
                    constraint: HashSet::new()
                },
                Attribute{
                    name: String::from("LastName"),
                    data_type: AttributeType::VarChar(255),
                    constraint: HashSet::new()
                },
                Attribute{
                    name: String::from("FirstName"),
                    data_type: AttributeType::VarChar(255),
                    constraint: HashSet::new()
                },
                Attribute{
                    name: String::from("Address"),
                    data_type: AttributeType::VarChar(255),
                    constraint: HashSet::new()
                },
                Attribute{
                    name: String::from("City"),
                    data_type: AttributeType::VarChar(255),
                    constraint: HashSet::new()
                },
            ],
            primary_key: None,
        };

        let values = HashMap::new();

        let actual = table.insert(&values);

        assert_eq!(actual, None);
    }
}