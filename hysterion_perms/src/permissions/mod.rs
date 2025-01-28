// External crate imports
use serde::{Deserialize, Serialize};
use sqlx::Row;
use pumpkin::command::{PermissionChecker, register_permission_checker};
use pumpkin::plugin::api::context::Context;
use std::sync::Arc;
use uuid::Uuid;
use tokio::runtime::Runtime;

// Internal crate imports
use crate::db::get_db;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub permissions: Vec<String>,
    pub level: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPermissions {
    pub uuid: String,
    pub roles: Vec<String>,
    pub direct_permissions: Vec<String>,
}

impl PlayerPermissions {
    #[allow(dead_code)]
    pub async fn has_permission(&self, permission: &str) -> bool {
        // Check direct permissions first
        if self.direct_permissions.contains(&permission.to_string()) {
            return true;
        }

        // Check role permissions
        for role_name in &self.roles {
            if let Ok(role) = get_role(role_name).await {
                if role.permissions.contains(&permission.to_string()) {
                    return true;
                }
            }
        }

        false
    }
}

#[allow(dead_code)]
pub async fn init_tables() -> Result<(), sqlx::Error> {
    let db = get_db().await;
    
    // Create roles table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS roles (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            permissions TEXT NOT NULL,
            level INTEGER NOT NULL
        )"
    )
    .execute(&db.pool)
    .await?;

    // Create player_roles table (many-to-many relationship)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS player_roles (
            id INTEGER PRIMARY KEY,
            player_uuid TEXT NOT NULL,
            role_name TEXT NOT NULL,
            FOREIGN KEY(role_name) REFERENCES roles(name)
        )"
    )
    .execute(&db.pool)
    .await?;

    // Create player_permissions table (direct permissions)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS player_permissions (
            id INTEGER PRIMARY KEY,
            player_uuid TEXT NOT NULL,
            permission TEXT NOT NULL
        )"
    )
    .execute(&db.pool)
    .await?;

    Ok(())
}

#[allow(dead_code)]
pub async fn create_role(name: &str, level: i32) -> Result<(), sqlx::Error> {
    let db = get_db().await;
    
    // Use INSERT OR REPLACE to handle existing roles
    sqlx::query(
        "INSERT OR REPLACE INTO roles (name, permissions, level) VALUES ($1, $2, $3)"
    )
    .bind(name)
    .bind("[]") // Empty permissions array
    .bind(level)
    .execute(&db.pool)
    .await?;

    Ok(())
}

#[allow(dead_code)]
pub async fn get_role(name: &str) -> Result<Role, sqlx::Error> {
    let db = get_db().await;
    
    let row = sqlx::query("SELECT * FROM roles WHERE name = $1")
        .bind(name)
        .fetch_one(&db.pool)
        .await?;

    let permissions: Vec<String> = serde_json::from_str(row.get("permissions"))
        .unwrap_or_default();

    Ok(Role {
        name: row.get("name"),
        permissions,
        level: i32::from(row.get::<i32, _>("level")),
    })
}

#[allow(dead_code)]
pub async fn add_role_permission(role_name: &str, permission: &str) -> Result<(), sqlx::Error> {
    let db = get_db().await;
    let mut role = get_role(role_name).await?;
    
    // Only add permission if it doesn't exist
    if !role.permissions.contains(&permission.to_string()) {
        role.permissions.push(permission.to_string());
        let permissions_json = serde_json::to_string(&role.permissions).unwrap();

        sqlx::query("UPDATE roles SET permissions = $1 WHERE name = $2")
            .bind(permissions_json)
            .bind(role_name)
            .execute(&db.pool)
            .await?;
    }

    Ok(())
}

#[allow(dead_code)]
pub async fn get_player_permissions(uuid: &str) -> Result<PlayerPermissions, sqlx::Error> {
    let db = get_db().await;
    
    // Get player roles
    let roles: Vec<String> = sqlx::query("SELECT role_name FROM player_roles WHERE player_uuid = $1")
        .bind(uuid)
        .fetch_all(&db.pool)
        .await?
        .into_iter()
        .map(|row| row.get("role_name"))
        .collect();

    // Get direct permissions
    let direct_permissions: Vec<String> = sqlx::query("SELECT permission FROM player_permissions WHERE player_uuid = $1")
        .bind(uuid)
        .fetch_all(&db.pool)
        .await?
        .into_iter()
        .map(|row| row.get("permission"))
        .collect();

    Ok(PlayerPermissions {
        uuid: uuid.to_string(),
        roles,
        direct_permissions,
    })
}

pub async fn add_player_to_role(uuid: &str, role_name: &str) -> Result<(), sqlx::Error> {
    let db = get_db().await;
    
    sqlx::query("INSERT INTO player_roles (player_uuid, role_name) VALUES ($1, $2)")
        .bind(uuid)
        .bind(role_name)
        .execute(&db.pool)
        .await?;

    Ok(())
}

pub async fn add_player_permission(uuid: &str, permission: &str) -> Result<(), sqlx::Error> {
    let db = get_db().await;
    
    sqlx::query("INSERT INTO player_permissions (player_uuid, permission) VALUES ($1, $2)")
        .bind(uuid)
        .bind(permission)
        .execute(&db.pool)
        .await?;

    Ok(())
}

pub struct HysterionPermissionChecker {
    runtime: &'static Runtime,
}

impl HysterionPermissionChecker {
    pub fn new() -> Self {
        Self {
            runtime: crate::get_runtime(),
        }
    }
}

impl PermissionChecker for HysterionPermissionChecker {
    fn check_permission(&self, uuid: &Uuid, permission: &str) -> bool {
        self.runtime.block_on(async {
            let uuid_str = uuid.to_string();
            
            match get_player_permissions(&uuid_str).await {
                Ok(player_perms) => {
                    if player_perms.direct_permissions.contains(&permission.to_string()) {
                        return true;
                    }

                    for role_name in &player_perms.roles {
                        if let Ok(role) = get_role(role_name).await {
                            if role.permissions.contains(&permission.to_string()) {
                                return true;
                            }
                        }
                    }
                    false
                },
                Err(e) => {
                    log::error!("Failed to check permissions for {}: {}", uuid_str, e);
                    false
                }
            }
        })
    }
}

pub fn init_permission_system(server: &Context) {
    let checker = Arc::new(HysterionPermissionChecker::new());
    register_permission_checker(checker);
} 