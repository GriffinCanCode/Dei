using System;
using System.Collections.Generic;
using System.Linq;

namespace ExampleProject
{
    /// <summary>
    /// Example of a god class with too many responsibilities
    /// </summary>
    public class UserManager
    {
        private readonly Database _database;
        private readonly EmailService _emailService;
        private readonly Logger _logger;
        private readonly Cache _cache;
        private readonly TokenProvider _tokenProvider;
        private readonly Validator _validator;
        
        // Authentication methods
        public User Login(string username, string password)
        {
            _logger.Log("Login attempt");
            if (ValidateCredentials(username, password))
            {
                var user = _database.GetUser(username);
                var token = GenerateToken(user);
                _cache.Store(token, user);
                return user;
            }
            throw new Exception("Invalid credentials");
        }
        
        public void Logout(string token)
        {
            _logger.Log("Logout");
            _cache.Remove(token);
            RevokeToken(token);
        }
        
        public bool ValidateCredentials(string username, string password)
        {
            var user = _database.GetUser(username);
            return user != null && user.Password == HashPassword(password);
        }
        
        public string GenerateToken(User user)
        {
            return _tokenProvider.CreateToken(user.Id, user.Role);
        }
        
        public void RefreshToken(string oldToken)
        {
            var user = _cache.Get(oldToken);
            var newToken = GenerateToken(user);
            _cache.Store(newToken, user);
            RevokeToken(oldToken);
        }
        
        public void RevokeToken(string token)
        {
            _tokenProvider.Revoke(token);
        }
        
        // Validation methods
        public bool ValidateEmail(string email)
        {
            return _validator.IsValidEmail(email);
        }
        
        public bool ValidatePassword(string password)
        {
            return password.Length >= 8 && password.Any(char.IsDigit);
        }
        
        public bool CheckPasswordStrength(string password)
        {
            var hasUpper = password.Any(char.IsUpper);
            var hasLower = password.Any(char.IsLower);
            var hasDigit = password.Any(char.IsDigit);
            var hasSpecial = password.Any(c => !char.IsLetterOrDigit(c));
            return hasUpper && hasLower && hasDigit && hasSpecial;
        }
        
        public bool ValidatePhoneNumber(string phone)
        {
            return phone.Length == 10 && phone.All(char.IsDigit);
        }
        
        public bool CheckEmailUnique(string email)
        {
            return _database.GetUserByEmail(email) == null;
        }
        
        public bool CheckUsernameUnique(string username)
        {
            return _database.GetUser(username) == null;
        }
        
        // Notification methods
        public void SendWelcomeEmail(User user)
        {
            var subject = "Welcome!";
            var body = $"Welcome {user.Username}!";
            _emailService.Send(user.Email, subject, body);
        }
        
        public void SendVerificationEmail(User user, string code)
        {
            var subject = "Verify your email";
            var body = $"Your code: {code}";
            _emailService.Send(user.Email, subject, body);
        }
        
        public void SendPasswordResetEmail(User user, string resetLink)
        {
            var subject = "Password Reset";
            var body = $"Click here: {resetLink}";
            _emailService.Send(user.Email, subject, body);
        }
        
        public void NotifyAccountLocked(User user)
        {
            var subject = "Account Locked";
            var body = "Your account has been locked due to suspicious activity.";
            _emailService.Send(user.Email, subject, body);
        }
        
        public void SendSecurityAlert(User user, string message)
        {
            var subject = "Security Alert";
            _emailService.Send(user.Email, subject, message);
        }
        
        // User management methods
        public User CreateUser(string username, string email, string password)
        {
            if (!ValidateEmail(email))
                throw new Exception("Invalid email");
            if (!ValidatePassword(password))
                throw new Exception("Weak password");
            if (!CheckEmailUnique(email))
                throw new Exception("Email taken");
                
            var user = new User
            {
                Username = username,
                Email = email,
                Password = HashPassword(password),
                CreatedAt = DateTime.UtcNow
            };
            
            _database.SaveUser(user);
            SendWelcomeEmail(user);
            _logger.Log($"User created: {username}");
            
            return user;
        }
        
        public void UpdateUser(User user)
        {
            _database.UpdateUser(user);
            _cache.Remove($"user_{user.Id}");
            _logger.Log($"User updated: {user.Id}");
        }
        
        public void DeleteUser(int userId)
        {
            var user = _database.GetUserById(userId);
            _database.DeleteUser(userId);
            _cache.Remove($"user_{userId}");
            _logger.Log($"User deleted: {userId}");
        }
        
        public User GetUser(int userId)
        {
            var cached = _cache.Get($"user_{userId}");
            if (cached != null)
                return cached;
                
            var user = _database.GetUserById(userId);
            _cache.Store($"user_{userId}", user);
            return user;
        }
        
        public List<User> SearchUsers(string query)
        {
            return _database.SearchUsers(query);
        }
        
        public void UpdateUserRole(int userId, string role)
        {
            var user = GetUser(userId);
            user.Role = role;
            UpdateUser(user);
            _logger.Log($"Role updated: {userId} -> {role}");
        }
        
        // Helper methods
        private string HashPassword(string password)
        {
            // Simplified hashing
            return Convert.ToBase64String(System.Text.Encoding.UTF8.GetBytes(password));
        }
    }
    
    // Supporting classes
    public class User
    {
        public int Id { get; set; }
        public string Username { get; set; }
        public string Email { get; set; }
        public string Password { get; set; }
        public string Role { get; set; }
        public DateTime CreatedAt { get; set; }
    }
    
    public class Database
    {
        public User GetUser(string username) => null;
        public User GetUserById(int id) => null;
        public User GetUserByEmail(string email) => null;
        public void SaveUser(User user) { }
        public void UpdateUser(User user) { }
        public void DeleteUser(int id) { }
        public List<User> SearchUsers(string query) => new();
    }
    
    public class EmailService
    {
        public void Send(string to, string subject, string body) { }
    }
    
    public class Logger
    {
        public void Log(string message) { }
    }
    
    public class Cache
    {
        public User Get(string key) => null;
        public void Store(string key, User value) { }
        public void Remove(string key) { }
    }
    
    public class TokenProvider
    {
        public string CreateToken(int userId, string role) => "";
        public void Revoke(string token) { }
    }
    
    public class Validator
    {
        public bool IsValidEmail(string email) => true;
    }
}

