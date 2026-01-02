package com.example;

import java.util.*;
import java.io.*;

/**
 * Example of a God Class anti-pattern in Java
 */
public class GodClass {
    private String name;
    private int age;
    private List<String> items;
    private Map<String, Object> cache;
    private boolean isActive;
    private double balance;
    private Connection dbConnection;
    private Logger logger;
    private Config config;
    private UserService userService;

    public GodClass(String name) {
        this.name = name;
        this.items = new ArrayList<>();
        this.cache = new HashMap<>();
    }

    // Too many responsibilities - user management
    public void createUser(String username, String email) {
        if (username == null || username.isEmpty()) {
            throw new IllegalArgumentException("Invalid username");
        }
        if (email == null || !email.contains("@")) {
            throw new IllegalArgumentException("Invalid email");
        }
        // Complex user creation logic
        for (int i = 0; i < 10; i++) {
            if (cache.containsKey(username)) {
                break;
            }
            processUserData(username, email);
        }
    }

    private void processUserData(String username, String email) {
        // Nested complexity
        if (config.isEnabled()) {
            if (userService.exists(username)) {
                for (String item : items) {
                    if (item.equals(username)) {
                        cache.put(username, email);
                        return;
                    }
                }
            }
        }
    }

    // Database operations - different responsibility
    public List<Object> queryDatabase(String sql) {
        List<Object> results = new ArrayList<>();
        try {
            if (dbConnection != null && sql != null) {
                for (int i = 0; i < 100; i++) {
                    if (i % 10 == 0) {
                        results.add(executeQuery(sql, i));
                    } else if (i % 5 == 0) {
                        results.add(executeQuery(sql, i * 2));
                    }
                }
            }
        } catch (Exception e) {
            logger.error("Query failed", e);
        }
        return results;
    }

    private Object executeQuery(String sql, int offset) {
        return null;
    }

    // Caching - another responsibility
    public void updateCache(String key, Object value) {
        if (key == null) return;
        
        switch (key.charAt(0)) {
            case 'a':
            case 'b':
            case 'c':
                cache.put(key, value);
                break;
            case 'd':
            case 'e':
                cache.put(key.toUpperCase(), value);
                break;
            default:
                if (value != null) {
                    cache.put(key, value.toString());
                }
        }
    }

    // Reporting - yet another responsibility
    public String generateReport() {
        StringBuilder report = new StringBuilder();
        
        for (Map.Entry<String, Object> entry : cache.entrySet()) {
            if (entry.getValue() != null) {
                if (entry.getKey().startsWith("user")) {
                    report.append("User: ").append(entry.getValue()).append("\n");
                } else if (entry.getKey().startsWith("item")) {
                    report.append("Item: ").append(entry.getValue()).append("\n");
                } else {
                    report.append("Other: ").append(entry.getKey()).append(" = ").append(entry.getValue()).append("\n");
                }
            }
        }
        
        return report.toString();
    }

    // Email sending - mixing concerns
    public void sendEmail(String to, String subject, String body) {
        if (to == null || to.isEmpty()) {
            throw new IllegalArgumentException("Invalid recipient");
        }
        
        for (int retry = 0; retry < 3; retry++) {
            try {
                if (config.getSmtpServer() != null) {
                    if (config.isSmtpEnabled()) {
                        // Send email logic
                        logger.info("Email sent to " + to);
                        return;
                    }
                }
            } catch (Exception e) {
                if (retry == 2) {
                    throw new RuntimeException("Failed to send email", e);
                }
            }
        }
    }

    // File operations - more mixing
    public void saveToFile(String filename) {
        if (filename == null) return;
        
        try (BufferedWriter writer = new BufferedWriter(new FileWriter(filename))) {
            for (String item : items) {
                if (item != null && !item.isEmpty()) {
                    writer.write(item);
                    writer.newLine();
                }
            }
        } catch (IOException e) {
            logger.error("Failed to save file", e);
        }
    }

    // Getters/Setters
    public String getName() { return name; }
    public void setName(String name) { this.name = name; }
    public int getAge() { return age; }
    public void setAge(int age) { this.age = age; }
    public boolean isActive() { return isActive; }
    public void setActive(boolean active) { isActive = active; }
    public double getBalance() { return balance; }
    public void setBalance(double balance) { this.balance = balance; }
}

// Stub classes for compilation
class Connection {}
class Logger {
    void error(String msg, Exception e) {}
    void info(String msg) {}
}
class Config {
    boolean isEnabled() { return true; }
    String getSmtpServer() { return "smtp.example.com"; }
    boolean isSmtpEnabled() { return true; }
}
class UserService {
    boolean exists(String username) { return false; }
}
