use schemars::{
    schema::{
        ArrayValidation, InstanceType, Schema, SchemaObject, SingleOrVec, SubschemaValidation,
    },
    JsonSchema,
};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

/// Generates a placeholder JSON value mirroring the shape of `T`.
/// Primitives become their type name (e.g. `"string"`), objects become
/// `{"field": ...}`, and arrays become `[...]`.
pub fn representation_of<T: JsonSchema>() -> Value {
    let root = schemars::schema_for!(T);
    schema_to_value(&root.schema.into(), &root.definitions)
}

fn schema_to_value(schema: &Schema, defs: &BTreeMap<String, Schema>) -> Value {
    match schema {
        Schema::Bool(_) => json!("any"),
        Schema::Object(obj) => schema_object_to_value(obj, defs),
    }
}

fn schema_object_to_value(obj: &SchemaObject, defs: &BTreeMap<String, Schema>) -> Value {
    // Resolve $ref
    if let Some(reference) = &obj.reference {
        if let Some(name) = reference.strip_prefix("#/definitions/") {
            if let Some(def) = defs.get(name) {
                return schema_to_value(def, defs);
            }
        }
        return json!("any");
    }

    // Object with named properties
    if let Some(object_validation) = &obj.object {
        let mut map = Map::new();
        for (key, prop) in &object_validation.properties {
            map.insert(key.clone(), schema_to_value(prop, defs));
        }
        return Value::Object(map);
    }

    // Array
    if let Some(array_validation) = &obj.array {
        return array_to_value(array_validation, defs);
    }

    // Unions (oneOf / anyOf / allOf)
    if let Some(sub) = &obj.subschemas {
        if let Some(v) = subschema_to_value(sub, defs) {
            return v;
        }
    }

    // Primitive
    if let Some(t) = &obj.instance_type {
        return primitive_to_value(t);
    }

    json!("any")
}

fn array_to_value(arr: &ArrayValidation, defs: &BTreeMap<String, Schema>) -> Value {
    let item = match &arr.items {
        Some(SingleOrVec::Single(item)) => schema_to_value(item, defs),
        Some(SingleOrVec::Vec(items)) => items
            .first()
            .map(|s| schema_to_value(s, defs))
            .unwrap_or(json!("any")),
        None => json!("any"),
    };
    Value::Array(vec![item])
}

fn subschema_to_value(sub: &SubschemaValidation, defs: &BTreeMap<String, Schema>) -> Option<Value> {
    let variants = sub
        .one_of
        .as_ref()
        .or(sub.any_of.as_ref())
        .or(sub.all_of.as_ref())?;
    variants.first().map(|s| schema_to_value(s, defs))
}

fn primitive_to_value(t: &SingleOrVec<InstanceType>) -> Value {
    let ty = match t {
        SingleOrVec::Single(x) => Some(x.as_ref()),
        // prefer the first non-null type (handles Option<T>)
        SingleOrVec::Vec(v) => v.iter().find(|x| **x != InstanceType::Null),
    };
    match ty {
        Some(InstanceType::String) => json!("string"),
        Some(InstanceType::Integer) => json!("integer"),
        Some(InstanceType::Number) => json!("number"),
        Some(InstanceType::Boolean) => json!("boolean"),
        Some(InstanceType::Array) => json!([]),
        Some(InstanceType::Object) => json!({}),
        _ => json!("any"),
    }
}
