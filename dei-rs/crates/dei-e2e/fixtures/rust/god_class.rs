//! God class example - does way too much

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};

/// A massive class that handles everything - BAD DESIGN
pub struct MegaUserManager {
    users: HashMap<u64, User>,
    sessions: HashMap<String, u64>,
    permissions: HashMap<u64, Vec<String>>,
    audit_log: Vec<String>,
    config: Config,
    cache: HashMap<String, String>,
    db_connection: Option<String>,
    email_queue: Vec<Email>,
    notification_settings: HashMap<u64, NotificationPrefs>,
    rate_limiter: HashMap<u64, RateLimit>,
}

impl MegaUserManager {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            sessions: HashMap::new(),
            permissions: HashMap::new(),
            audit_log: Vec::new(),
            config: Config::default(),
            cache: HashMap::new(),
            db_connection: None,
            email_queue: Vec::new(),
            notification_settings: HashMap::new(),
            rate_limiter: HashMap::new(),
        }
    }

    // Authentication methods
    pub fn authenticate(&mut self, username: &str, password: &str) -> Result<String, String> {
        let user = self.users.values()
            .find(|u| u.username == username)
            .ok_or("User not found")?;
        
        if self.verify_password(password, &user.password_hash) {
            let session_token = self.generate_session_token();
            self.sessions.insert(session_token.clone(), user.id);
            self.log_audit(&format!("User {} logged in", username));
            Ok(session_token)
        } else {
            Err("Invalid password".to_string())
        }
    }

    pub fn logout(&mut self, session_token: &str) -> Result<(), String> {
        if let Some(user_id) = self.sessions.remove(session_token) {
            self.log_audit(&format!("User {} logged out", user_id));
            Ok(())
        } else {
            Err("Invalid session".to_string())
        }
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> bool {
        // Simplified password verification
        password.len() > 8
    }

    pub fn generate_session_token(&self) -> String {
        format!("token_{}", self.sessions.len())
    }

    pub fn refresh_session(&mut self, token: &str) -> Result<String, String> {
        let user_id = self.sessions.get(token).ok_or("Invalid session")?;
        let new_token = self.generate_session_token();
        self.sessions.remove(token);
        self.sessions.insert(new_token.clone(), *user_id);
        Ok(new_token)
    }

    // User management methods
    pub fn create_user(&mut self, username: String, email: String, password: String) -> Result<u64, String> {
        if !self.validate_email(&email) {
            return Err("Invalid email".to_string());
        }
        if !self.validate_password_strength(&password) {
            return Err("Weak password".to_string());
        }
        let id = self.users.len() as u64 + 1;
        let user = User {
            id,
            username: username.clone(),
            email: email.clone(),
            password_hash: self.hash_password(&password),
        };
        self.users.insert(id, user);
        self.log_audit(&format!("Created user {}", username));
        self.send_welcome_email(&email);
        Ok(id)
    }

    pub fn update_user(&mut self, id: u64, username: Option<String>, email: Option<String>) -> Result<(), String> {
        let user = self.users.get_mut(&id).ok_or("User not found")?;
        if let Some(new_username) = username {
            user.username = new_username;
        }
        if let Some(new_email) = email {
            if !self.validate_email(&new_email) {
                return Err("Invalid email".to_string());
            }
            user.email = new_email;
        }
        self.log_audit(&format!("Updated user {}", id));
        Ok(())
    }

    pub fn delete_user(&mut self, id: u64) -> Result<(), String> {
        self.users.remove(&id).ok_or("User not found")?;
        self.sessions.retain(|_, user_id| *user_id != id);
        self.permissions.remove(&id);
        self.notification_settings.remove(&id);
        self.rate_limiter.remove(&id);
        self.log_audit(&format!("Deleted user {}", id));
        Ok(())
    }

    pub fn get_user(&self, id: u64) -> Option<&User> {
        self.users.get(&id)
    }

    pub fn list_users(&self) -> Vec<&User> {
        self.users.values().collect()
    }

    pub fn search_users(&self, query: &str) -> Vec<&User> {
        self.users.values()
            .filter(|u| u.username.contains(query) || u.email.contains(query))
            .collect()
    }

    // Permission methods
    pub fn grant_permission(&mut self, user_id: u64, permission: String) -> Result<(), String> {
        let perms = self.permissions.entry(user_id).or_insert_with(Vec::new);
        if !perms.contains(&permission) {
            perms.push(permission.clone());
            self.log_audit(&format!("Granted {} to user {}", permission, user_id));
        }
        Ok(())
    }

    pub fn revoke_permission(&mut self, user_id: u64, permission: &str) -> Result<(), String> {
        if let Some(perms) = self.permissions.get_mut(&user_id) {
            perms.retain(|p| p != permission);
            self.log_audit(&format!("Revoked {} from user {}", permission, user_id));
        }
        Ok(())
    }

    pub fn has_permission(&self, user_id: u64, permission: &str) -> bool {
        self.permissions.get(&user_id)
            .map(|perms| perms.contains(&permission.to_string()))
            .unwrap_or(false)
    }

    // Email methods
    pub fn send_welcome_email(&mut self, email: &str) {
        let email_obj = Email {
            to: email.to_string(),
            subject: "Welcome!".to_string(),
            body: "Welcome to our service".to_string(),
        };
        self.email_queue.push(email_obj);
    }

    pub fn send_password_reset(&mut self, email: &str) {
        let email_obj = Email {
            to: email.to_string(),
            subject: "Password Reset".to_string(),
            body: "Reset your password".to_string(),
        };
        self.email_queue.push(email_obj);
    }

    pub fn process_email_queue(&mut self) -> Result<(), String> {
        for email in self.email_queue.drain(..) {
            // Simulate sending
            println!("Sending email to {}", email.to);
        }
        Ok(())
    }

    // Notification methods
    pub fn update_notification_settings(&mut self, user_id: u64, prefs: NotificationPrefs) {
        self.notification_settings.insert(user_id, prefs);
        self.log_audit(&format!("Updated notification settings for user {}", user_id));
    }

    pub fn send_notification(&self, user_id: u64, message: &str) -> Result<(), String> {
        let prefs = self.notification_settings.get(&user_id)
            .ok_or("No preferences found")?;
        if prefs.enabled {
            println!("Notifying user {}: {}", user_id, message);
        }
        Ok(())
    }

    // Rate limiting methods
    pub fn check_rate_limit(&mut self, user_id: u64) -> Result<(), String> {
        let limit = self.rate_limiter.entry(user_id).or_insert(RateLimit::new());
        if limit.is_exceeded() {
            Err("Rate limit exceeded".to_string())
        } else {
            limit.increment();
            Ok(())
        }
    }

    pub fn reset_rate_limit(&mut self, user_id: u64) {
        self.rate_limiter.remove(&user_id);
    }

    // Cache methods
    pub fn cache_get(&self, key: &str) -> Option<&String> {
        self.cache.get(key)
    }

    pub fn cache_set(&mut self, key: String, value: String) {
        self.cache.insert(key, value);
    }

    pub fn cache_clear(&mut self) {
        self.cache.clear();
    }

    // Audit methods
    pub fn log_audit(&mut self, message: &str) {
        self.audit_log.push(format!("[{}] {}", self.get_timestamp(), message));
    }

    pub fn get_audit_log(&self) -> &[String] {
        &self.audit_log
    }

    pub fn export_audit_log(&self, path: &str) -> Result<(), String> {
        let mut file = File::create(path).map_err(|e| e.to_string())?;
        for entry in &self.audit_log {
            file.write_all(entry.as_bytes()).map_err(|e| e.to_string())?;
            file.write_all(b"\n").map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    // Validation helpers
    pub fn validate_email(&self, email: &str) -> bool {
        email.contains('@') && email.contains('.')
    }

    pub fn validate_password_strength(&self, password: &str) -> bool {
        password.len() >= 8
    }

    pub fn hash_password(&self, password: &str) -> String {
        format!("hash_{}", password)
    }

    // Config methods
    pub fn update_config(&mut self, key: &str, value: String) {
        self.config.settings.insert(key.to_string(), value);
    }

    pub fn get_config(&self, key: &str) -> Option<&String> {
        self.config.settings.get(key)
    }

    // Database methods
    pub fn connect_database(&mut self, connection_string: String) -> Result<(), String> {
        self.db_connection = Some(connection_string);
        self.log_audit("Connected to database");
        Ok(())
    }

    pub fn disconnect_database(&mut self) {
        self.db_connection = None;
        self.log_audit("Disconnected from database");
    }

    // Helper methods
    fn get_timestamp(&self) -> String {
        "2024-01-01T00:00:00Z".to_string()
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

#[derive(Debug)]
pub struct Email {
    pub to: String,
    pub subject: String,
    pub body: String,
}

#[derive(Debug)]
pub struct NotificationPrefs {
    pub enabled: bool,
}

#[derive(Debug)]
pub struct RateLimit {
    count: u32,
    max: u32,
}

impl RateLimit {
    fn new() -> Self {
        Self { count: 0, max: 100 }
    }

    fn is_exceeded(&self) -> bool {
        self.count >= self.max
    }

    fn increment(&mut self) {
        self.count += 1;
    }
}

#[derive(Debug, Default)]
pub struct Config {
    settings: HashMap<String, String>,
}

