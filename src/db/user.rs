use rusqlite::Connection;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub display_name: Option<String>,
    pub created_at: i64,
    pub created_by: Option<i64>,
}

impl User {
    pub fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
        Ok(User {
            id: row.get("id")?,
            name: row.get("name")?,
            display_name: row.get("display_name")?,
            created_at: row.get("created_at")?,
            created_by: row.get("created_by")?,
        })
    }
}

/// Creates a new user in the database.
pub fn create_user(
    conn: &Connection,
    name: &str,
    display_name: Option<&str>,
    created_by: Option<i64>,
) -> Result<i64, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    conn.execute(
        "INSERT INTO users (name, display_name, created_at, created_by) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![name, display_name, now, created_by],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE constraint failed") {
            format!("User '{}' already exists", name)
        } else {
            e.to_string()
        }
    })?;

    let user_id = conn.last_insert_rowid();
    Ok(user_id)
}

/// Retrieves a user by name.
pub fn get_user_by_name(conn: &Connection, name: &str) -> Result<Option<User>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, display_name, created_at, created_by FROM users WHERE name = ?1")
        .map_err(|e| e.to_string())?;

    let user = stmt
        .query_row([name], User::from_row)
        .map(Some)
        .or_else(|e| {
            if e == rusqlite::Error::QueryReturnedNoRows {
                Ok(None)
            } else {
                Err(e.to_string())
            }
        })?;

    Ok(user)
}

/// Retrieves a user by ID.
pub fn get_user_by_id(conn: &Connection, id: i64) -> Result<Option<User>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, display_name, created_at, created_by FROM users WHERE id = ?1")
        .map_err(|e| e.to_string())?;

    let user = stmt
        .query_row([id], User::from_row)
        .map(Some)
        .or_else(|e| {
            if e == rusqlite::Error::QueryReturnedNoRows {
                Ok(None)
            } else {
                Err(e.to_string())
            }
        })?;

    Ok(user)
}

/// Lists all users.
pub fn list_users(conn: &Connection) -> Result<Vec<User>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, display_name, created_at, created_by FROM users ORDER BY name")
        .map_err(|e| e.to_string())?;

    let users = stmt
        .query_map([], User::from_row)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(users)
}

/// Deletes a user by name. Returns error if user has tasks or namespace memberships.
pub fn delete_user(conn: &Connection, name: &str) -> Result<(), String> {
    // First get the user ID
    let user = get_user_by_name(conn, name)?
        .ok_or_else(|| format!("User '{}' not found", name))?;

    // Check if user owns any tasks
    let task_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM items WHERE owner_id = ?1 OR assignee_id = ?1",
            [user.id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if task_count > 0 {
        return Err(format!(
            "Cannot delete user '{}': has {} associated task(s). Reassign or delete tasks first.",
            name, task_count
        ));
    }

    // Delete user (CASCADE will remove user_namespaces entries)
    let deleted = conn
        .execute("DELETE FROM users WHERE id = ?1", [user.id])
        .map_err(|e| e.to_string())?;

    if deleted == 0 {
        return Err(format!("User '{}' not found", name));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::get_test_conn;

    #[test]
    fn test_create_and_get_user() {
        let (conn, _temp_file) = get_test_conn();

        let user_id = create_user(&conn, "testuser", Some("Test User"), None).unwrap();
        assert!(user_id > 0);

        let user = get_user_by_name(&conn, "testuser").unwrap().unwrap();
        assert_eq!(user.name, "testuser");
        assert_eq!(user.display_name, Some("Test User".to_string()));
    }

    #[test]
    fn test_create_duplicate_user() {
        let (conn, _temp_file) = get_test_conn();

        create_user(&conn, "duplicate", None, None).unwrap();
        let result = create_user(&conn, "duplicate", None, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    #[test]
    fn test_list_users() {
        let (conn, _temp_file) = get_test_conn();

        // Should have default user from init
        let users = list_users(&conn).unwrap();
        assert!(!users.is_empty());
    }

    #[test]
    fn test_delete_user() {
        let (conn, _temp_file) = get_test_conn();

        create_user(&conn, "todelete", None, None).unwrap();
        let result = delete_user(&conn, "todelete");
        assert!(result.is_ok());

        let user = get_user_by_name(&conn, "todelete").unwrap();
        assert!(user.is_none());
    }
}
