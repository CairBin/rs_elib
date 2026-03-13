pub fn validate_password(password: &str) -> Result<(), &'static str>{
    if password.len() < 8 {
        return Err("Password length must be between 8 and 16 characters.");
    }

    if !password.chars().any(|c| c.is_uppercase()) {
        return Err("Password must contain at least one uppercase letter.");
    }

    if !password.chars().any(|c| c.is_lowercase()) {
        return Err("Password must contain at least one lowercase letter.");
    }

    if !password.chars().any(|c| c.is_numeric()) {
        return Err("Password must contain at least one numeric digit.");
    }

    if !password.chars().any(|c| "!@#$%^&*()_+-=[]{};':\"\\|,.<>/?".contains(c)) {
        return Err("Password must contain at least one special character (!@#$%^&*()_+-=[]{};':\"\\|,.<>/?).");
    }

    Ok(())
}