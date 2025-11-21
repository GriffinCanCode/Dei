//! Healthy, well-structured Rust code

use std::collections::HashMap;

/// A simple, focused user repository
pub struct UserRepository {
    users: HashMap<u64, User>,
}

impl UserRepository {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    pub fn add(&mut self, user: User) -> Result<(), String> {
        if self.users.contains_key(&user.id) {
            return Err("User already exists".to_string());
        }
        self.users.insert(user.id, user);
        Ok(())
    }

    pub fn get(&self, id: u64) -> Option<&User> {
        self.users.get(&id)
    }

    pub fn remove(&mut self, id: u64) -> Option<User> {
        self.users.remove(&id)
    }

    pub fn count(&self) -> usize {
        self.users.len()
    }
}

#[derive(Clone, Debug)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

impl User {
    pub fn new(id: u64, name: String, email: String) -> Self {
        Self { id, name, email }
    }

    pub fn validate_email(&self) -> bool {
        self.email.contains('@')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_repository() {
        let mut repo = UserRepository::new();
        let user = User::new(1, "Alice".to_string(), "alice@example.com".to_string());
        assert!(repo.add(user).is_ok());
        assert_eq!(repo.count(), 1);
    }
}

