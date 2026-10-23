use anyhow::{bail, Result};

pub fn is_python_version_or_greater(version: &str, min_minor_version: i32) -> Result<bool> {
    let version_parts = split_version(version)?;

    if version_parts.1 >= min_minor_version {
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
}
