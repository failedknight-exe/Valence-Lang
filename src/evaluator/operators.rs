//! Evaluation of arithmetic, comparison, boolean, and concatenation operators.

use super::value::Value;

pub fn apply_op(left: Value, op: &str, right: Value) -> Value {
    if op == "EqualEqual" {
        return Value::Boolean(left == right);
    }
    if op == "NotEqual" {
        return Value::Boolean(left != right);
    }

    if op == "And" {
        return match (&left, &right) {
            (Value::Boolean(l), Value::Boolean(r)) => Value::Boolean(*l && *r),
            _ => Value::Null,
        };
    }
    if op == "Or" {
        return match (&left, &right) {
            (Value::Boolean(l), Value::Boolean(r)) => Value::Boolean(*l || *r),
            _ => Value::Null,
        };
    }

    if op == "And" {
        return match (&left, &right) {
            (Value::Boolean(l), Value::Boolean(r)) => Value::Boolean(*l && *r),
            _ => Value::Null,
        };
    }
    if op == "Or" {
        return match (&left, &right) {
            (Value::Boolean(l), Value::Boolean(r)) => Value::Boolean(*l || *r),
            _ => Value::Null,
        };
    }
    match (&left, &right) {
        (Value::Integer(l), Value::Integer(r)) => match op {
            "Plus" => Value::Integer(l + r),
            "Minus" => Value::Integer(l - r),
            "Star" => Value::Integer(l * r),
            "Slash" => if *r == 0 { Value::Error("Division by zero. You tried to split the universe by nothing. Bold move, terrible math.".to_string()) } else { Value::Integer(l / r) },
            "Percent" => if *r == 0 { Value::Error("Modulo by zero. Zero is not a divisor; it's a personality flaw in arithmetic.".to_string()) } else { Value::Integer(l % r) },
            "DoubleStar" => if *r < 0 { Value::Null } else { Value::Integer(l.pow(*r as u32)) },
            "EqualEqual" => Value::Boolean(l == r),
            "NotEqual" => Value::Boolean(l != r),
            "Greater" => Value::Boolean(l > r),
            "Less" => Value::Boolean(l < r),
            "GreaterEqual" => Value::Boolean(l >= r),
            "LessEqual" => Value::Boolean(l <= r),
            _ => Value::Null,
        },
        (Value::Float(l), Value::Float(r)) => match op {
            "Plus" => Value::Float(l + r),
            "Minus" => Value::Float(l - r),
            "Star" => Value::Float(l * r),
            "Slash" => if *r == 0.0 { Value::Error("Division by zero. You tried to split the universe by nothing. Bold move, terrible math.".to_string()) } else { Value::Float(l / r) },
            "Percent" => if *r == 0.0 { Value::Error("Modulo by zero. Zero is not a divisor; it's a personality flaw in arithmetic.".to_string()) } else { Value::Float(l % r) },
            "DoubleStar" => Value::Float(l.powf(*r)),
            "EqualEqual" => Value::Boolean(l == r),
            "NotEqual" => Value::Boolean(l != r),
            "Greater" => Value::Boolean(l > r),
            "Less" => Value::Boolean(l < r),
            "GreaterEqual" => Value::Boolean(l >= r),
            "LessEqual" => Value::Boolean(l <= r),
            _ => Value::Null,
        },
        (Value::Integer(l), Value::Float(r)) => {
            let lf = *l as f64;
            match op {
                "Plus" => Value::Float(lf + r),
                "Minus" => Value::Float(lf - r),
                "Star" => Value::Float(lf * r),
                "Slash" => if *r == 0.0 { Value::Error("Division by zero. You tried to split the universe by nothing. Bold move, terrible math.".to_string()) } else { Value::Float(lf / r) },
                "Percent" => if *r == 0.0 { Value::Error("Modulo by zero. Zero is not a divisor; it's a personality flaw in arithmetic.".to_string()) } else { Value::Float(lf % r) },
                "DoubleStar" => Value::Float(lf.powf(*r)),
                "EqualEqual" => Value::Boolean(lf == *r),
                "NotEqual" => Value::Boolean(lf != *r),
                "Greater" => Value::Boolean(lf > *r),
                "Less" => Value::Boolean(lf < *r),
                "GreaterEqual" => Value::Boolean(lf >= *r),
                "LessEqual" => Value::Boolean(lf <= *r),
                _ => Value::Null,
            }
        }
        (Value::Float(l), Value::Integer(r)) => {
            let rf = *r as f64;
            match op {
                "Plus" => Value::Float(l + rf),
                "Minus" => Value::Float(l - rf),
                "Star" => Value::Float(l * rf),
                "Slash" => if rf == 0.0 { Value::Error("Division by zero. You tried to split the universe by nothing. Bold move, terrible math.".to_string()) } else { Value::Float(l / rf) },
                "Percent" => if rf == 0.0 { Value::Error("Modulo by zero. Zero is not a divisor; it's a personality flaw in arithmetic.".to_string()) } else { Value::Float(l % rf) },
                "DoubleStar" => Value::Float(l.powf(rf)),
                "EqualEqual" => Value::Boolean(*l == rf),
                "NotEqual" => Value::Boolean(*l != rf),
                "Greater" => Value::Boolean(*l > rf),
                "Less" => Value::Boolean(*l < rf),
                "GreaterEqual" => Value::Boolean(*l >= rf),
                "LessEqual" => Value::Boolean(*l <= rf),
                _ => Value::Null,
            }
        }
        (Value::StringVal(l), Value::StringVal(r)) => match op {
            "Plus" => Value::StringVal(format!("{}{}", l, r)),
            "EqualEqual" => Value::Boolean(l == r),
            "NotEqual" => Value::Boolean(l != r),
            _ => Value::Null,
        },
        (Value::StringVal(l), other) => {
            if op == "Plus" { Value::StringVal(format!("{}{}", l, other)) } else { Value::Null }
        }
        (other, Value::StringVal(r)) => {
            if op == "Plus" { Value::StringVal(format!("{}{}", other, r)) } else { Value::Null }
        }
        (Value::Boolean(l), Value::Boolean(r)) => match op {
            "EqualEqual" => Value::Boolean(l == r),
            "NotEqual" => Value::Boolean(l != r),
            _ => Value::Null,
        },
        _ => Value::Null,
    }
}