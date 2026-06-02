use crate::blackboard::BlackboardResource;

pub fn evaluate_condition(expr: &str, bb: &BlackboardResource) -> bool {
    let expr = expr.trim();
    let parts: Vec<&str> = expr.split("&&").collect();
    for part in parts {
        if !evaluate_single_condition(part.trim(), bb) {
            return false;
        }
    }
    true
}

fn evaluate_single_condition(expr: &str, bb: &BlackboardResource) -> bool {
    if expr.contains(">=") {
        let parts: Vec<&str> = expr.splitn(2, ">=").collect();
        let key = parts[0].trim();
        let val: f64 = parts[1].trim().parse().unwrap_or(0.0);
        return bb.get(key).unwrap_or(0.0) >= val;
    }
    if expr.contains("<=") {
        let parts: Vec<&str> = expr.splitn(2, "<=").collect();
        let key = parts[0].trim();
        let val: f64 = parts[1].trim().parse().unwrap_or(0.0);
        return bb.get(key).unwrap_or(0.0) <= val;
    }
    if expr.contains(">") {
        let parts: Vec<&str> = expr.splitn(2, '>').collect();
        let key = parts[0].trim();
        let val: f64 = parts[1].trim().parse().unwrap_or(0.0);
        return bb.get(key).unwrap_or(0.0) > val;
    }
    if expr.contains("<") {
        let parts: Vec<&str> = expr.splitn(2, '<').collect();
        let key = parts[0].trim();
        let val: f64 = parts[1].trim().parse().unwrap_or(0.0);
        return bb.get(key).unwrap_or(0.0) < val;
    }
    if expr.contains("==") {
        let parts: Vec<&str> = expr.splitn(2, "==").collect();
        let key = parts[0].trim();
        let val: f64 = parts[1].trim().parse().unwrap_or(0.0);
        return bb.get(key).unwrap_or(0.0) == val;
    }
    true
}