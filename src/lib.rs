use std::fmt::Display;

use once_cell::sync::Lazy;
use regex::Regex;

static FIELD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[_a-z][_a-z0-9]*(\.[_a-z][_a-z0-9]*)*").unwrap());

#[derive(Debug, Clone)]
struct Field(String);
impl Field {
    fn new(name: impl Into<String>) -> Result<Self, ParseQueryError> {
        let name = name.into();
        if !FIELD_REGEX.is_match(&name) {
            return Err(ParseQueryError(format!(
                "failed to parse field_name: {name}"
            )));
        }
        Ok(Self(name))
    }
}
impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone)]
enum Order {
    Asc(Field),
    Desc(Field),
}
impl Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Asc(field) => f.write_fmt(format_args!("{field} asc")),
            Self::Desc(field) => f.write_fmt(format_args!("{field} desc")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrderBy(Vec<Order>);
impl OrderBy {
    pub fn new(query: impl Into<String>) -> Result<Self, ParseQueryError> {
        query
            .into()
            .as_str()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(str::trim)
            .map(|s| {
                let mut s = s.split(' ').filter(|s| !s.is_empty());
                (s.next(), s.next())
            })
            .map(|query| match query {
                (Some(field), None) => Ok(Order::Asc(Field::new(field)?)),
                (Some(field), Some(order)) if order == "asc" || order == "ASC" => {
                    Ok(Order::Asc(Field::new(field)?))
                }
                (Some(field), Some(order)) if order == "desc" || order == "DESC" => {
                    Ok(Order::Desc(Field::new(field)?))
                }
                (field, order) => Err(ParseQueryError(format!(
                    "failed to parse {query:?}: field({field:?}, order({order:?})"
                ))),
            })
            .collect::<Result<Vec<Order>, ParseQueryError>>()
            .map(Self)
    }
}
impl Display for OrderBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let orders_string = self
            .0
            .iter()
            .fold(Vec::new(), |mut acc, cur| {
                acc.push(cur.to_string());
                acc
            })
            .join(",");
        if orders_string.is_empty() {
            f.write_str("")
        } else {
            f.write_fmt(format_args!("order_by {orders_string}"))
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseQueryError(String);
impl Display for ParseQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod test {
    use proptest::{
        string::string_regex,
        test_runner::{Config, TestCaseError::Reject, TestRunner},
    };

    use super::*;

    const ORDER_BY_QUERY_REGEX: &str = r"[_a-z][_a-z0-9]*(\.[_a-z][_a-z0-9]*)*( +(asc|desc))?( *, *[_a-z][_a-z0-9]*(\.[_a-z][_a-z0-9]*)*( +(asc|desc))?)*";

    #[test]
    fn test_parse_arbitrary() {
        let result = TestRunner::new(Config::with_cases(100_000)).run(
            &string_regex(ORDER_BY_QUERY_REGEX).unwrap(),
            |query| {
                OrderBy::new(query)
                    .map(|_| ())
                    .map_err(|e| Reject(e.to_string().into()))
            },
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_empty() {
        let order_by = OrderBy::new("");
        assert!(order_by.is_ok());
        let order_by = order_by.unwrap();
        assert!(order_by.0.is_empty());
        assert_eq!(order_by.to_string(), "".to_string());
    }
}
