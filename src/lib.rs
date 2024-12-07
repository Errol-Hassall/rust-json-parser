mod tokenize;
use std::collections::HashMap;

pub enum Value {
    /// literal characters `null`
    Null,
    /// true or false json values
    Boolean(bool),
    /// anything surrounded by a quote is a string
    String(String),
    /// numbers stored as a 64 bit float
    Number(f64),
    /// zero to many json values
    Array(Vec<Value>),
    /// string keys with json values
    Object(HashMap<String, Value>),
}
