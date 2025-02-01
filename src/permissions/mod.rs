// External crate imports
use serde::{Deserialize, Serialize};
use sqlx::Row;
use pumpkin::plugin::api::{Context, PermissionChecker};
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
    pub uuid: Uuid,
    pub roles: Vec<String>,
    pub direct_permissions: Vec<String>,
}

impl PlayerPermissions {
    pub async fn has_permission(&self, permission: &str) -> bool {
        log::info!("[HysterionPerms] Starting permission check for {}: {}", self.uuid, permission);
        
        // Check direct permissions first
        log::info!("[HysterionPerms] Direct permissions: {:?}", self.direct_permissions);
        for direct_perm in &self.direct_permissions {
            if check_permission_match(direct_perm, permission) {
                log::info!(
                    "[HysterionPerms] Direct permission match: '{}' matches '{}'",
                    direct_perm,
                    permission
                );
                return true;
            }
        }

        // Check role permissions
        log::info!("[HysterionPerms] Checking roles: {:?}", self.roles);
        for role_name in &self.roles {
            match get_role(role_name).await {
                Ok(role) => {
                    log::info!("[HysterionPerms] Role '{}' permissions: {:?}", role_name, role.permissions);
                    for role_perm in &role.permissions {
                        if check_permission_match(role_perm, permission) {
                            log::info!(
                                "[HysterionPerms] Role permission match: '{}' from role '{}' matches '{}'",
                                role_perm,
                                role_name,
                                permission
                            );
                            return true;
                        }
                    }
                },
                Err(e) => log::error!("[HysterionPerms] Failed to get role {}: {}", role_name, e),
            }
        }

        log::info!("[HysterionPerms] No matching permissions found for '{}'", permission);
        false
    }
}

// Helper function to check if a permission matches, including wildcard support
fn check_permission_match(held_permission: &str, required_permission: &str) -> bool {
    let matches = held_permission == "*" || 
                 held_permission == required_permission || 
                 (held_permission.ends_with(".*") && required_permission.starts_with(&held_permission[..held_permission.len()-2]));
    
    log::info!(
        "[HysterionPerms] Permission match check: '{}' against '{}' = {}",
        held_permission,
        required_permission,
        matches
    );
    
    matches
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

pub async fn get_player_permissions(uuid: &Uuid) -> Result<PlayerPermissions, sqlx::Error> {
    let db = get_db().await;
    let uuid_str = uuid.to_string();
    
    // Get player roles
    let roles: Vec<String> = sqlx::query("SELECT role_name FROM player_roles WHERE player_uuid = $1")
        .bind(&uuid_str)
        .fetch_all(&db.pool)
        .await?
        .into_iter()
        .map(|row| row.get("role_name"))
        .collect();

    // Get direct permissions
    let direct_permissions: Vec<String> = sqlx::query("SELECT permission FROM player_permissions WHERE player_uuid = $1")
        .bind(&uuid_str)
        .fetch_all(&db.pool)
        .await?
        .into_iter()
        .map(|row| row.get("permission"))
        .collect();

    Ok(PlayerPermissions {
        uuid: *uuid,
        roles,
        direct_permissions,
    })
}

pub async fn add_player_to_role(uuid: &Uuid, role_name: &str) -> Result<(), sqlx::Error> {
    let db = get_db().await;
    let uuid_str = uuid.to_string();
    
    sqlx::query("INSERT INTO player_roles (player_uuid, role_name) VALUES ($1, $2)")
        .bind(&uuid_str)
        .bind(role_name)
        .execute(&db.pool)
        .await?;

    Ok(())
}

pub async fn add_player_permission(uuid: &Uuid, permission: &str) -> Result<(), sqlx::Error> {
    let db = get_db().await;
    let uuid_str = uuid.to_string();
    
    sqlx::query("INSERT INTO player_permissions (player_uuid, permission) VALUES ($1, $2)")
        .bind(&uuid_str)
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
            log::info!("[HysterionPerms] Checking permission '{}' for player {}", permission, uuid);
            
            match get_player_permissions(uuid).await {
                Ok(player_perms) => {
                    log::info!("[HysterionPerms] Found permissions for player: {:?}", player_perms);
                    player_perms.has_permission(permission).await
                },
                Err(e) => {
                    log::error!("[HysterionPerms] Failed to check permissions for {}: {}", uuid, e);
                    false
                }
            }
        })
    }
}

pub async fn init_permission_system(server: &Context) {
    log::info!("[HysterionPerms] Initializing permission system");
    let checker = Arc::new(HysterionPermissionChecker::new());
    server.register_permission_checker(checker).await;
    log::info!("[HysterionPerms] Permission system initialized and checker registered");
} 