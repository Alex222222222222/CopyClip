use clip::{Clip, SearchConstraint};
use rusqlite::{params, params_from_iter, Connection, Params, ParamsFromIter};
use tauri::async_runtime::Mutex;

use crate::database::label_name_to_table_name;

const DEFAULT_SEARCH_LIMIT: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// The type of SQL statement
enum SQLStatementType {
    Where,
    Limit,
    Join,
}

/// Convert a list of search constraints to a SQL query
fn sql_statement_type_of_search_constraint(
    search_constraint: &SearchConstraint,
) -> SQLStatementType {
    match search_constraint {
        SearchConstraint::TextContains(_) => SQLStatementType::Where,
        SearchConstraint::TextRegex(_) => SQLStatementType::Where,
        SearchConstraint::TextFuzzy(_) => SQLStatementType::Where,
        SearchConstraint::TimestampGreaterThan(_) => SQLStatementType::Where,
        SearchConstraint::TimestampLessThan(_) => SQLStatementType::Where,
        SearchConstraint::HasLabel(_) => SQLStatementType::Join,
        SearchConstraint::Limit(_) => SQLStatementType::Limit,
    }
}

/// Convert a search constraint to a SQL query
/// The pos is the position of the search constraint + 1 in the query
/// pos is used to bind the parameters to the query
fn search_constraint_to_sql(search_constraint: &SearchConstraint, pos: u64) -> String {
    match search_constraint {
        SearchConstraint::TextContains(_) => format!("text LIKE '%?{}%'", pos),
        SearchConstraint::TextRegex(_) => format!("regexp(text, ?{})", pos),
        SearchConstraint::TextFuzzy(_) => format!("fuzzy_search(text, ?{})", pos),
        SearchConstraint::TimestampGreaterThan(_) => format!("timestamp > ?{}", pos),
        SearchConstraint::TimestampLessThan(_) => format!("timestamp < ?{}", pos),
        SearchConstraint::HasLabel(label) => {
            let table_name = label_name_to_table_name(label);
            format!("INNER JOIN {} ON clips.id = {}.id;", table_name, table_name)
        }
        SearchConstraint::Limit(_) => format!("LIMIT ?{}", pos),
    }
}

/// Convert a search constraint to a rusqlite::types::Value
fn search_constraint_to_value(search_constraint: &SearchConstraint) -> rusqlite::types::Value {
    match search_constraint {
        SearchConstraint::TextContains(text) => rusqlite::types::Value::Text(text.clone()),
        SearchConstraint::TextRegex(regex) => rusqlite::types::Value::Text(regex.clone()),
        SearchConstraint::TextFuzzy(fuzzy) => rusqlite::types::Value::Text(fuzzy.clone()),
        SearchConstraint::TimestampGreaterThan(timestamp) => {
            rusqlite::types::Value::Integer(*timestamp)
        }
        SearchConstraint::TimestampLessThan(timestamp) => {
            rusqlite::types::Value::Integer(*timestamp)
        }
        SearchConstraint::HasLabel(_) => rusqlite::types::Value::Null,
        SearchConstraint::Limit(limit) => rusqlite::types::Value::Integer(*limit as i64),
    }
}

/// Group the constraints by the type of SQL statement
/// The first element of the tuple is the where clause
/// The second element of the tuple is the limit clause
/// The third element of the tuple is the join clause
fn group_constraints_by_sql_statement_type(
    constraints: &Vec<SearchConstraint>,
) -> (
    Vec<&SearchConstraint>,
    Vec<&SearchConstraint>,
    Vec<&SearchConstraint>,
) {
    let mut where_constraints = Vec::new();
    let mut limit_constraints = Vec::new();
    let mut join_constraints = Vec::new();

    for constraint in constraints {
        match sql_statement_type_of_search_constraint(&constraint) {
            SQLStatementType::Where => where_constraints.push(constraint),
            SQLStatementType::Limit => limit_constraints.push(constraint),
            SQLStatementType::Join => join_constraints.push(constraint),
        }
    }

    (where_constraints, limit_constraints, join_constraints)
}

/// Verify the validity of limit constraints
/// There should be at most one limit constraint, if not the first one is used
/// If there is no limit constraint, the default limit is used
fn verify_limit_constraints(constraints: &Vec<&SearchConstraint>) -> SearchConstraint {
    if constraints.len() == 0 {
        SearchConstraint::Limit(DEFAULT_SEARCH_LIMIT)
    } else {
        constraints[0].clone()
    }
}

/// Convert a list of search constraints to a SQL query
fn search_constraints_to_sql(constraints: &Vec<SearchConstraint>) -> String {
    let (where_constraints, limit_constraints, join_constraints) =
        group_constraints_by_sql_statement_type(constraints);
    let limit_constraint = verify_limit_constraints(&limit_constraints);

    let mut query = String::from(
        "SELECT id, type, data, search_text, timestamp FROM clips ORDER BY timestamp DESC",
    );
    query.push('\n');

    for (i, constraint) in where_constraints.iter().enumerate() {
        if i == 0 {
            query.push_str(" WHERE ");
        } else {
            query.push_str(" AND ");
        }

        query.push_str(&search_constraint_to_sql(constraint, i as u64 + 1));
        query.push('\n');
    }

    query.push_str(&search_constraint_to_sql(
        &limit_constraint,
        where_constraints.len() as u64 + 1,
    ));
    query.push('\n');

    for (i, constraint) in join_constraints.iter().enumerate() {
        query.push_str(&search_constraint_to_sql(constraint, i as u64 + 1));
        query.push('\n');
    }

    query
}

/// Convert a list of search constraints to a list of rusqlite::types::Value
fn search_constraints_to_params(
    constraints: &Vec<SearchConstraint>,
) -> Vec<rusqlite::types::Value> {
    let (where_constraints, limit_constraints, _) =
        group_constraints_by_sql_statement_type(constraints);
    let limit_constraint = verify_limit_constraints(&limit_constraints);

    let mut params = Vec::new();

    for constraint in where_constraints {
        params.push(search_constraint_to_value(&constraint));
    }

    params.push(search_constraint_to_value(&limit_constraint));

    params
}

/// search the database for clips that match the search constraints
pub async fn search_clips(
    connection: &Mutex<Connection>,
    constraints: Vec<SearchConstraint>,
) -> Result<Vec<clip::Clip>, anyhow::Error> {
    let params = search_constraints_to_params(&constraints);
    let params = params_from_iter(params.iter());

    let connection = connection.lock().await;

    let mut stmt = connection.prepare(&search_constraints_to_sql(&constraints))?;
    let clips = stmt
        .query_map(params, Clip::from_database_row)?
        .collect::<Result<Vec<clip::Clip>, _>>()?;

    Ok(clips)
}
