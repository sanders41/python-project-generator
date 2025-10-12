use anyhow::{bail, Result};

pub fn is_python_version_or_greater(version: &str, min_minor_version: i32) -> Result<bool> {
    let version_parts = split_version(version)?;

    if version_parts.1 >= min_minor_version {
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(feature = "fastapi")]
pub fn is_allowed_fastapi_python_version(version: &str) -> Result<bool> {
    let version_parts = split_version(version)?;

    if version_parts.0 >= 3 && version_parts.1 >= 11 {
        Ok(true)
    } else {
        Ok(false)
    }
}

fn split_version(version: &str) -> Result<(i32, i32)> {
    let split_version: Vec<&str> = version.split('.').collect();
    if split_version.len() < 2 {
        bail!("Major and minor versions not found");
    }

    let major = split_version[0].parse::<i32>()?;
    let minor = split_version[1].parse::<i32>()?;

    Ok((major, minor))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_312() {
        let result = is_python_version_or_greater("3.12", 11).unwrap();
        assert!(result);
    }

    #[test]
    fn test_python_311_311() {
        let result = is_python_version_or_greater("3.11", 11).unwrap();
        assert!(result);
    }

    #[test]
    fn test_python_311_310() {
        let result = is_python_version_or_greater("3.10", 11).unwrap();
        assert!(!result);
    }

    #[cfg(feature = "fastapi")]
    #[test]
    fn test_is_allowed_fastapi_python_version() {
        let result = is_allowed_fastapi_python_version("3.11").unwrap();
        assert!(result);
    }

    #[cfg(feature = "fastapi")]
    #[test]
    fn test_is_unallowed_major_fastapi_python_version() {
        let result = is_allowed_fastapi_python_version("2.11").unwrap();
        assert!(!result);
    }

    #[cfg(feature = "fastapi")]
    #[test]
    fn test_is_unallowed_minor_fastapi_python_version() {
        let result = is_allowed_fastapi_python_version("3.10").unwrap();
        assert!(!result);
    }
}
