//! Shared expression and query-reference compilation.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::kernel::JsonPointer;
use crate::pack::error::{PackError, PackResult};
use crate::pack::refs::PackRef;

/// Structural query reference used by rule and recipe packs.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub(crate) struct CompiledQueryRef {
    pub pack: PackRef,
    pub id: String,
}

/// Structural expression tree for score-model compilation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) enum CompiledExpr {
    ConstInt(i64),
    ConstBool(bool),
    Field(JsonPointer),
    Exists(JsonPointer),
    IsNull(JsonPointer),
    Add(Vec<CompiledExpr>),
    Sub(Vec<CompiledExpr>),
    Mul(Vec<CompiledExpr>),
    Div(Vec<CompiledExpr>),
    Max(Vec<CompiledExpr>),
    Min(Vec<CompiledExpr>),
    Eq(Box<CompiledExpr>, Box<CompiledExpr>),
    Ne(Box<CompiledExpr>, Box<CompiledExpr>),
    Gt(Box<CompiledExpr>, Box<CompiledExpr>),
    Ge(Box<CompiledExpr>, Box<CompiledExpr>),
    Lt(Box<CompiledExpr>, Box<CompiledExpr>),
    Le(Box<CompiledExpr>, Box<CompiledExpr>),
    And(Vec<CompiledExpr>),
    Or(Vec<CompiledExpr>),
    Not(Box<CompiledExpr>),
}

/// Compiles one expression subtree from JSON.
pub(crate) fn compile_expr(
    pack_id: &str,
    path: &JsonPointer,
    value: &Value,
) -> PackResult<CompiledExpr> {
    let object = value
        .as_object()
        .ok_or_else(|| expression_error(pack_id, path, "expression must be an object"))?;
    let op = object
        .get("op")
        .and_then(Value::as_str)
        .ok_or_else(|| expression_error(pack_id, path, "expression op must be a string"))?;

    match op {
        "const_int" => compile_const_int(pack_id, path, object.get("value")),
        "const_bool" => compile_const_bool(pack_id, path, object.get("value")),
        "field" => compile_pointer_op(pack_id, path, object.get("path"), CompiledExpr::Field),
        "exists" => compile_pointer_op(pack_id, path, object.get("path"), CompiledExpr::Exists),
        "is_null" => compile_pointer_op(pack_id, path, object.get("path"), CompiledExpr::IsNull),
        "add" => compile_variadic_expr(pack_id, path, object.get("args"), CompiledExpr::Add),
        "sub" => compile_variadic_expr(pack_id, path, object.get("args"), CompiledExpr::Sub),
        "mul" => compile_variadic_expr(pack_id, path, object.get("args"), CompiledExpr::Mul),
        "div" => compile_variadic_expr(pack_id, path, object.get("args"), CompiledExpr::Div),
        "max" => compile_variadic_expr(pack_id, path, object.get("args"), CompiledExpr::Max),
        "min" => compile_variadic_expr(pack_id, path, object.get("args"), CompiledExpr::Min),
        "and" => compile_variadic_expr(pack_id, path, object.get("args"), CompiledExpr::And),
        "or" => compile_variadic_expr(pack_id, path, object.get("args"), CompiledExpr::Or),
        "eq" => compile_binary_expr(
            pack_id,
            path,
            object.get("lhs"),
            object.get("rhs"),
            CompiledExpr::Eq,
        ),
        "ne" => compile_binary_expr(
            pack_id,
            path,
            object.get("lhs"),
            object.get("rhs"),
            CompiledExpr::Ne,
        ),
        "gt" => compile_binary_expr(
            pack_id,
            path,
            object.get("lhs"),
            object.get("rhs"),
            CompiledExpr::Gt,
        ),
        "ge" => compile_binary_expr(
            pack_id,
            path,
            object.get("lhs"),
            object.get("rhs"),
            CompiledExpr::Ge,
        ),
        "lt" => compile_binary_expr(
            pack_id,
            path,
            object.get("lhs"),
            object.get("rhs"),
            CompiledExpr::Lt,
        ),
        "le" => compile_binary_expr(
            pack_id,
            path,
            object.get("lhs"),
            object.get("rhs"),
            CompiledExpr::Le,
        ),
        "not" => compile_unary_expr(pack_id, path, object.get("arg"), CompiledExpr::Not),
        other => Err(expression_error(
            pack_id,
            path,
            format!("unsupported expression operator `{other}`"),
        )),
    }
}

/// Compiles one structural query reference object from JSON.
pub(crate) fn compile_query_ref(
    pack_id: &str,
    path: &JsonPointer,
    value: &Value,
) -> PackResult<CompiledQueryRef> {
    let object = value
        .as_object()
        .ok_or_else(|| expression_error(pack_id, path, "query ref must be an object"))?;
    let pack = object
        .get("pack")
        .and_then(Value::as_str)
        .ok_or_else(|| expression_error(pack_id, path, "query ref pack must be a string"))?;
    let id = object
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| expression_error(pack_id, path, "query ref id must be a string"))?;
    if id.is_empty() {
        return Err(expression_error(
            pack_id,
            path,
            "query ref id must be non-empty",
        ));
    }

    Ok(CompiledQueryRef {
        pack: PackRef::parse(pack)?,
        id: id.to_owned(),
    })
}

fn compile_const_int(
    pack_id: &str,
    path: &JsonPointer,
    value: Option<&Value>,
) -> PackResult<CompiledExpr> {
    let number = value
        .and_then(Value::as_i64)
        .ok_or_else(|| expression_error(pack_id, path, "const_int requires integer value"))?;
    Ok(CompiledExpr::ConstInt(number))
}

fn compile_const_bool(
    pack_id: &str,
    path: &JsonPointer,
    value: Option<&Value>,
) -> PackResult<CompiledExpr> {
    let boolean = value
        .and_then(Value::as_bool)
        .ok_or_else(|| expression_error(pack_id, path, "const_bool requires boolean value"))?;
    Ok(CompiledExpr::ConstBool(boolean))
}

fn compile_pointer_op(
    pack_id: &str,
    path: &JsonPointer,
    value: Option<&Value>,
    constructor: fn(JsonPointer) -> CompiledExpr,
) -> PackResult<CompiledExpr> {
    let pointer = value.and_then(Value::as_str).ok_or_else(|| {
        expression_error(pack_id, path, "pointer expression requires string path")
    })?;
    let pointer = JsonPointer::parse(pointer).map_err(|error| {
        expression_error(pack_id, path, format!("invalid JSON Pointer: {error}"))
    })?;
    Ok(constructor(pointer))
}

fn compile_variadic_expr(
    pack_id: &str,
    path: &JsonPointer,
    value: Option<&Value>,
    constructor: fn(Vec<CompiledExpr>) -> CompiledExpr,
) -> PackResult<CompiledExpr> {
    let items = value.and_then(Value::as_array).ok_or_else(|| {
        expression_error(pack_id, path, "variadic expression requires args array")
    })?;
    if items.is_empty() {
        return Err(expression_error(
            pack_id,
            path,
            "variadic expression requires at least one arg",
        ));
    }

    let mut compiled = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        compiled.push(compile_expr(
            pack_id,
            &path.push_token("args").push_token(&index.to_string()),
            item,
        )?);
    }
    Ok(constructor(compiled))
}

fn compile_binary_expr(
    pack_id: &str,
    path: &JsonPointer,
    lhs: Option<&Value>,
    rhs: Option<&Value>,
    constructor: fn(Box<CompiledExpr>, Box<CompiledExpr>) -> CompiledExpr,
) -> PackResult<CompiledExpr> {
    let lhs = compile_expr(
        pack_id,
        &path.push_token("lhs"),
        lhs.ok_or_else(|| expression_error(pack_id, path, "binary expression requires lhs"))?,
    )?;
    let rhs = compile_expr(
        pack_id,
        &path.push_token("rhs"),
        rhs.ok_or_else(|| expression_error(pack_id, path, "binary expression requires rhs"))?,
    )?;
    Ok(constructor(Box::new(lhs), Box::new(rhs)))
}

fn compile_unary_expr(
    pack_id: &str,
    path: &JsonPointer,
    arg: Option<&Value>,
    constructor: fn(Box<CompiledExpr>) -> CompiledExpr,
) -> PackResult<CompiledExpr> {
    let arg = compile_expr(
        pack_id,
        &path.push_token("arg"),
        arg.ok_or_else(|| expression_error(pack_id, path, "unary expression requires arg"))?,
    )?;
    Ok(constructor(Box::new(arg)))
}

fn expression_error(pack_id: &str, path: &JsonPointer, reason: impl Into<String>) -> PackError {
    PackError::ExpressionCompile {
        pack_id: pack_id.to_owned(),
        path: path.clone(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{compile_expr, compile_query_ref, CompiledExpr};
    use crate::kernel::JsonPointer;

    #[test]
    fn compiles_nested_expressions() {
        let expr = compile_expr(
            "generic/lift-v2",
            &JsonPointer::parse("/lift_score").expect("pointer"),
            &json!({
                "op": "add",
                "args": [
                    {"op": "const_int", "value": 1},
                    {"op": "field", "path": "/touch/edit_files"}
                ]
            }),
        )
        .expect("expr");

        assert!(matches!(expr, CompiledExpr::Add(args) if args.len() == 2));
    }

    #[test]
    fn rejects_bad_pointer_and_unknown_query_ref() {
        let expr_error = compile_expr(
            "generic/lift-v2",
            &JsonPointer::parse("/lift_score").expect("pointer"),
            &json!({"op": "field", "path": "touch/edit_files"}),
        )
        .expect_err("bad pointer should fail");
        assert!(matches!(
            expr_error,
            crate::pack::error::PackError::ExpressionCompile { .. }
        ));

        let query_error = compile_query_ref(
            "generic/policy",
            &JsonPointer::parse("/rules/0/query").expect("pointer"),
            &json!({"pack": "not-a-ref", "id": "use_statement"}),
        )
        .expect_err("bad query ref should fail");
        assert!(matches!(
            query_error,
            crate::pack::error::PackError::InvalidPackRef { .. }
        ));
    }
}
