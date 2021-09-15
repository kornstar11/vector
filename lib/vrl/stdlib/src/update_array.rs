use std::collections::BTreeMap;
use vrl::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct UpdateArray;

impl Function for UpdateArray {
    fn identifier(&self) -> &'static str {
        "update_array"
    }

    fn parameters(&self) -> &'static [Parameter] {
        &[
            Parameter {
                keyword: "value",
                kind: kind::ARRAY,
                required: true,
            },
            Parameter {
                keyword: "update",
                kind: kind::OBJECT,
                required: true,
            },
        ]
    }

    fn examples(&self) -> &'static [Example] {
        &[
            Example {
                title: "update array",
                source: r#"update_array([{"a": 1}, {"a": 2}], {"b": 3})"#,
                result: Ok(r#"[{"a": 1, "b": 3}, {"a": 2, "b": 3}]"#),
            },
            Example {
                title: "update non object array",
                source: r#"update_array([1, {"a": 2}], {"b": 3})"#,
                result: Ok(r#"[1, {"a": 2, "b": 3}]"#),
            },
        ]
    }

    fn compile(&self, _state: &state::Compiler, mut arguments: ArgumentList) -> Compiled {
        let value = arguments.required("value");
        let update = arguments.required("update");
        Ok(Box::new(UpdateArrayFn { value, update }))
    }
}

#[derive(Debug, Clone)]
struct UpdateArrayFn {
    value: Box<dyn Expression>,
    update: Box<dyn Expression>,
}

impl Expression for UpdateArrayFn {
    fn resolve(&self, ctx: &mut Context) -> Resolved {
        match self.value.resolve(ctx)? {
            Value::Array(arr) => match self.update.resolve(ctx)? {
                Value::Object(map) => Ok(Value::Array(update_array(arr, map))),
                value => Err(value::Error::Expected {
                    got: value.kind(),
                    expected: Kind::Object,
                }
                .into()),
            },
            value => Err(value::Error::Expected {
                got: value.kind(),
                expected: Kind::Array,
            }
            .into()),
        }
    }

    fn type_def(&self, _state: &state::Compiler) -> TypeDef {
        TypeDef::new().array_mapped::<(), Kind>(map! { (): Kind::all() })
    }
}

fn update_array(arr: Vec<Value>, update: BTreeMap<String, Value>) -> Vec<Value> {
    arr.into_iter()
        .map(|v| {
            if let Value::Object(mut existing) = v {
                existing.extend(update.clone());
                Value::Object(existing)
            } else {
                v
            }
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    test_function![
        update_array => UpdateArray;

        non_object_array {
            args: func_args![value: value!([42]), update: value!({"b": 1})],
            want: Ok(value!([42])),
            tdef: TypeDef::new().array_mapped::<(), Kind>(map! { (): Kind::all() }),
        }

        object_array {
            args: func_args![value: value!([{"a": 1}, {"a": 2}]), update: value!({"b": 3})],
            want: Ok(value!([{"a": 1, "b": 3}, {"a": 2, "b": 3}])),
            tdef: TypeDef::new().array_mapped::<(), Kind>(map! { (): Kind::all() }),
        }

        object_array_nested {
            args: func_args![value: value!([{"a": 1}, {"a": 2}]), update: value!({"b": {"c": 3, "d": 4}})],
            want: Ok(value!([{"a": 1, "b": {"c": 3, "d": 4}}, {"a": 2, "b": {"c": 3, "d": 4}}])),
            tdef: TypeDef::new().array_mapped::<(), Kind>(map! { (): Kind::all() }),
        }

        mixed_array {
            args: func_args![value: value!([42, {"a": 1}]), update: value!({"b": 3})],
            want: Ok(value!([42, {"a": 1, "b": 3}])),
            tdef: TypeDef::new().array_mapped::<(), Kind>(map! { (): Kind::all() }),
        }
    ];
}
