use rusqlite::Connection;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct Namespace {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: i64,
    pub created_by: Option<i64>,
}

impl Namespace {
    pub fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
        Ok(Namespace {
            id: row.get("id")?,
            name: row.get("name")?,
            description: row.get("description")?,
            created_at: row.get("created_at")?,
            created_by: row.get("created_by")?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct NamespaceMembership {
    pub user_id: i64,
    pub namespace_id: i64,
    pub role: String,
    pub created_at: i64,
    pub user_name: Option<String>,
}

/// Creates a new namespace in the database.
pub fn create_namespace(
    conn: &Connection,
    name: &str,
    description: Option<&str>,
    created_by: i64,
) -> Result<i64, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    conn.execute(
        "INSERT INTO namespaces (name, description, created_at, created_by) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![name, description, now, created_by],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE constraint failed") {
            format!("Namespace '{}' already exists", name)
        } else {
            e.to_string()
        }
    })?;

    let namespace_id = conn.last_insert_rowid();

    // Auto-add creator as owner
    conn.execute(
        "INSERT INTO user_namespaces (user_id, namespace_id, role, created_at) VALUES (?1, ?2, 'owner', ?3)",
        rusqlite::params![created_by, namespace_id, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(namespace_id)
}

/// Retrieves a namespace by name.
pub fn get_namespace_by_name(conn: &Connection, name: &str) -> Result<Option<Namespace>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, description, created_at, created_by FROM namespaces WHERE name = ?1")
        .map_err(|e| e.to_string())?;

    let namespace = stmt
        .query_row([name], Namespace::from_row)
        .map(Some)
        .or_else(|e| {
            if e == rusqlite::Error::QueryReturnedNoRows {
                Ok(None)
            } else {
                Err(e.to_string())
            }
        })?;

    Ok(namespace)
}

/// Retrieves a namespace by ID.
pub fn get_namespace_by_id(conn: &Connection, id: i64) -> Result<Option<Namespace>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, description, created_at, created_by FROM namespaces WHERE id = ?1")
        .map_err(|e| e.to_string())?;

    let namespace = stmt
        .query_row([id], Namespace::from_row)
        .map(Some)
        .or_else(|e| {
            if e == rusqlite::Error::QueryReturnedNoRows {
                Ok(None)
            } else {
                Err(e.to_string())
            }
        })?;

    Ok(namespace)
}

/// Lists all namespaces.
pub fn list_namespaces(conn: &Connection) -> Result<Vec<Namespace>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, description, created_at, created_by FROM namespaces ORDER BY name")
        .map_err(|e| e.to_string())?;

    let namespaces = stmt
        .query_map([], Namespace::from_row)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(namespaces)
}

/// Deletes a namespace by name. Returns error if namespace has tasks.
pub fn delete_namespace(conn: &Connection, name: &str) -> Result<(), String> {
    // Cannot delete the default namespace
    if name == "default" {
        return Err("Cannot delete the 'default' namespace".to_string());
    }

    let namespace = get_namespace_by_name(conn, name)?
        .ok_or_else(|| format!("Namespace '{}' not found", name))?;

    // Check if namespace has tasks
    let task_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM items WHERE namespace_id = ?1",
            [namespace.id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if task_count > 0 {
        return Err(format!(
            "Cannot delete namespace '{}': has {} task(s). Move or delete tasks first.",
            name, task_count
        ));
    }

    // Delete namespace (CASCADE will remove user_namespaces entries)
    let deleted = conn
        .execute("DELETE FROM namespaces WHERE id = ?1", [namespace.id])
        .map_err(|e| e.to_string())?;

    if deleted == 0 {
        return Err(format!("Namespace '{}' not found", name));
    }

    Ok(())
}

/// Adds a user to a namespace with the specified role.
pub fn add_user_to_namespace(
    conn: &Connection,
    namespace_name: &str,
    user_name: &str,
    role: &str,
) -> Result<(), String> {
    // Validate role
    let valid_roles = ["owner", "admin", "member", "viewer"];
    if !valid_roles.contains(&role) {
        return Err(format!(
            "Invalid role '{}'. Must be one of: {}",
            role,
            valid_roles.join(", ")
        ));
    }

    // Get namespace
    let namespace = get_namespace_by_name(conn, namespace_name)?
        .ok_or_else(|| format!("Namespace '{}' not found", namespace_name))?;

    // Get user
    let user = crate::db::user::get_user_by_name(conn, user_name)?
        .ok_or_else(|| format!("User '{}' not found", user_name))?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Insert or update membership
    conn.execute(
        "INSERT INTO user_namespaces (user_id, namespace_id, role, created_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(user_id, namespace_id) DO UPDATE SET role = ?3",
        rusqlite::params![user.id, namespace.id, role, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Removes a user from a namespace.
pub fn remove_user_from_namespace(
    conn: &Connection,
    namespace_name: &str,
    user_name: &str,
) -> Result<(), String> {
    // Get namespace
    let namespace = get_namespace_by_name(conn, namespace_name)?
        .ok_or_else(|| format!("Namespace '{}' not found", namespace_name))?;

    // Get user
    let user = crate::db::user::get_user_by_name(conn, user_name)?
        .ok_or_else(|| format!("User '{}' not found", user_name))?;

    // Check this isn't the last owner
    let owner_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM user_namespaces WHERE namespace_id = ?1 AND role = 'owner'",
            [namespace.id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let is_owner: bool = conn
        .query_row(
            "SELECT role = 'owner' FROM user_namespaces WHERE user_id = ?1 AND namespace_id = ?2",
            [user.id, namespace.id],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if is_owner && owner_count <= 1 {
        return Err(format!(
            "Cannot remove '{}' from '{}': they are the only owner. Assign another owner first.",
            user_name, namespace_name
        ));
    }

    let deleted = conn
        .execute(
            "DELETE FROM user_namespaces WHERE user_id = ?1 AND namespace_id = ?2",
            [user.id, namespace.id],
        )
        .map_err(|e| e.to_string())?;

    if deleted == 0 {
        return Err(format!(
            "User '{}' is not a member of namespace '{}'",
            user_name, namespace_name
        ));
    }

    Ok(())
}

/// Gets the role of a user in a namespace.
pub fn get_user_role(
    conn: &Connection,
    user_id: i64,
    namespace_id: i64,
) -> Result<Option<String>, String> {
    let role = conn
        .query_row(
            "SELECT role FROM user_namespaces WHERE user_id = ?1 AND namespace_id = ?2",
            [user_id, namespace_id],
            |row| row.get(0),
        )
        .map(Some)
        .or_else(|e| {
            if e == rusqlite::Error::QueryReturnedNoRows {
                Ok(None)
            } else {
                Err(e.to_string())
            }
        })?;

    Ok(role)
}

/// Lists all members of a namespace.
pub fn list_namespace_members(
    conn: &Connection,
    namespace_name: &str,
) -> Result<Vec<NamespaceMembership>, String> {
    let namespace = get_namespace_by_name(conn, namespace_name)?
        .ok_or_else(|| format!("Namespace '{}' not found", namespace_name))?;

    let mut stmt = conn
        .prepare(
            "SELECT un.user_id, un.namespace_id, un.role, un.created_at, u.name as user_name
             FROM user_namespaces un
             JOIN users u ON un.user_id = u.id
             WHERE un.namespace_id = ?1
             ORDER BY u.name",
        )
        .map_err(|e| e.to_string())?;

    let members = stmt
        .query_map([namespace.id], |row| {
            Ok(NamespaceMembership {
                user_id: row.get("user_id")?,
                namespace_id: row.get("namespace_id")?,
                role: row.get("role")?,
                created_at: row.get("created_at")?,
                user_name: row.get("user_name")?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(members)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::user::create_user;
    use crate::tests::get_test_conn;

    #[test]
    fn test_create_and_get_namespace() {
        let (conn, _temp_file) = get_test_conn();

        // Get default user ID
        let user = crate::db::user::get_user_by_name(&conn, &std::env::var("USER").unwrap_or("default".to_string()))
            .unwrap()
            .unwrap();

        let ns_id = create_namespace(&conn, "testns", Some("Test namespace"), user.id).unwrap();
        assert!(ns_id > 0);

        let ns = get_namespace_by_name(&conn, "testns").unwrap().unwrap();
        assert_eq!(ns.name, "testns");
        assert_eq!(ns.description, Some("Test namespace".to_string()));
    }

    #[test]
    fn test_create_duplicate_namespace() {
        let (conn, _temp_file) = get_test_conn();

        let user = crate::db::user::get_user_by_name(&conn, &std::env::var("USER").unwrap_or("default".to_string()))
            .unwrap()
            .unwrap();

        create_namespace(&conn, "dupns", None, user.id).unwrap();
        let result = create_namespace(&conn, "dupns", None, user.id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    #[test]
    fn test_add_remove_user_from_namespace() {
        let (conn, _temp_file) = get_test_conn();

        // Create a new user
        create_user(&conn, "newmember", None, None).unwrap();

        // Add to default namespace
        let result = add_user_to_namespace(&conn, "default", "newmember", "member");
        assert!(result.is_ok());

        // Remove from default namespace
        let result = remove_user_from_namespace(&conn, "default", "newmember");
        assert!(result.is_ok());
    }

    #[test]
    fn test_cannot_delete_default_namespace() {
        let (conn, _temp_file) = get_test_conn();

        let result = delete_namespace(&conn, "default");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot delete the 'default' namespace"));
    }

    #[test]
    fn test_list_namespace_members() {
        let (conn, _temp_file) = get_test_conn();

        let members = list_namespace_members(&conn, "default").unwrap();
        assert!(!members.is_empty());
        assert!(members.iter().any(|m| m.role == "owner"));
    }
}
